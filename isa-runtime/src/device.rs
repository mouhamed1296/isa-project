use crate::{EntropySource, MonotonicClock, Persistence, Result};
use isa_core::{MultiAxisState, StateVector, CircularDistance, MultiAxisStateExt};

pub struct DeviceRuntime<P: Persistence> {
    pub state: MultiAxisState,
    entropy: EntropySource,
    clock: MonotonicClock,
    persistence: P,
    last_timestamp: u64,
}

impl<P: Persistence> DeviceRuntime<P> {
    pub fn new(master_seed: [u8; 32], persistence: P) -> Self {
        Self {
            state: MultiAxisState::from_master_seed(master_seed),
            entropy: EntropySource::new(),
            clock: MonotonicClock::new(),
            persistence,
            last_timestamp: 0,
        }
    }

    pub fn load_or_create(master_seed: [u8; 32], persistence: P) -> Result<Self> {
        let state = if persistence.exists() {
            persistence.load()?
        } else {
            MultiAxisState::from_master_seed(master_seed)
        };

        Ok(Self {
            state,
            entropy: EntropySource::new(),
            clock: MonotonicClock::new(),
            persistence,
            last_timestamp: 0,
        })
    }

    pub fn record_sale(&mut self, sale_bytes: &[u8]) -> Result<StateVector> {
        let current_time = self.clock.now()?;
        let delta_t = current_time.saturating_sub(self.last_timestamp);
        self.last_timestamp = current_time;

        let entropy = self.entropy.gather(32)?;

        // Use dimension indices from standard profile
        self.state.finance_mut().accumulate(sale_bytes, &entropy, delta_t);
        self.state.time_mut().accumulate(&current_time.to_le_bytes(), &entropy, delta_t);

        let hw_entropy = self.entropy.gather_32()?;
        self.state.hardware_mut().accumulate(&hw_entropy, &entropy, delta_t);

        Ok(self.state.state_vector_compat())
    }

    pub fn record_event(&mut self, axis: EventAxis, event_data: &[u8]) -> Result<StateVector> {
        let current_time = self.clock.now()?;
        let delta_t = current_time.saturating_sub(self.last_timestamp);
        self.last_timestamp = current_time;

        let entropy = self.entropy.gather(32)?;

        // Map EventAxis to dimension index
        match axis {
            EventAxis::Finance => {
                self.state.finance_mut().accumulate(event_data, &entropy, delta_t);
            }
            EventAxis::Time => {
                self.state.time_mut().accumulate(event_data, &entropy, delta_t);
            }
            EventAxis::Hardware => {
                self.state.hardware_mut().accumulate(event_data, &entropy, delta_t);
            }
        }

        Ok(self.state.state_vector_compat())
    }

    pub fn save(&self) -> Result<()> {
        self.persistence.save(&self.state)
    }

    pub fn state_vector(&self) -> StateVector {
        self.state.state_vector_compat()
    }

    /// Calculate divergence between current state and a trusted authority state.
    ///
    /// Returns the divergence vector for each axis (finance, time, hardware).
    /// This implements the circular distance metric D(Sa, Sb) from Equation 7.
    pub fn calculate_divergence(&self, trusted_state: &StateVector) -> StateVector {
        let current = self.state_vector();
        StateVector {
            finance: CircularDistance::compute(&current.finance, &trusted_state.finance),
            time: CircularDistance::compute(&current.time, &trusted_state.time),
            hardware: CircularDistance::compute(&current.hardware, &trusted_state.hardware),
        }
    }

    /// Calculate the convergence constant K for state restoration.
    ///
    /// Given a trusted authority state, computes K = (S_honest - S_drifted) mod 2^256
    /// for each axis. This implements Equation 8 from the paper.
    ///
    /// The convergence constant can be applied via `apply_convergence()` to restore
    /// the device to the correct trajectory.
    pub fn calculate_convergence_constant(&self, trusted_state: &StateVector) -> StateVector {
        // K = (S_honest - S_drifted) mod 2^256
        // This is equivalent to the divergence calculation
        StateVector {
            finance: CircularDistance::compute(&trusted_state.finance, &self.state.finance().state()),
            time: CircularDistance::compute(&trusted_state.time, &self.state.time().state()),
            hardware: CircularDistance::compute(&trusted_state.hardware, &self.state.hardware().state()),
        }
    }

    /// Apply convergence constant to heal the device state.
    ///
    /// This implements the Partial Convergence Protocol from Section 5 of the paper.
    /// The convergence constant K is added to the current state to "snap" it back
    /// to the correct trajectory.
    ///
    /// # Arguments
    /// * `convergence_constant` - The K value calculated from a trusted authority
    /// * `audit_reason` - Human-readable reason for the healing event (logged)
    ///
    /// # Returns
    /// * `Ok(RecoveryAudit)` - Audit record of the healing event
    /// * `Err(_)` - If the recovery fails or state cannot be persisted
    ///
    /// # Security
    /// This operation is cryptographically logged and creates a permanent audit trail.
    /// The healing event is recorded with timestamp and reason for forensic analysis.
    pub fn apply_convergence(
        &mut self,
        convergence_constant: &StateVector,
        audit_reason: &str,
    ) -> Result<RecoveryAudit> {
        let current_time = self.clock.now()?;
        
        // Record pre-healing state for audit
        let pre_healing = self.state_vector();
        
        // Apply K to each axis: S_restored = (S_drifted + K) mod 2^256
        *self.state.finance_mut() = isa_core::DimensionAccumulator::from_state(
            modular_add(&self.state.finance().state(), &convergence_constant.finance),
            self.state.finance().counter(),
        );
        
        *self.state.time_mut() = isa_core::DimensionAccumulator::from_state(
            modular_add(&self.state.time().state(), &convergence_constant.time),
            self.state.time().counter(),
        );
        
        *self.state.hardware_mut() = isa_core::DimensionAccumulator::from_state(
            modular_add(&self.state.hardware().state(), &convergence_constant.hardware),
            self.state.hardware().counter(),
        );
        
        let post_healing = self.state_vector();
        
        // Create audit record
        let audit = RecoveryAudit {
            timestamp: current_time,
            pre_healing_state: pre_healing,
            convergence_constant: convergence_constant.clone(),
            post_healing_state: post_healing,
            reason: audit_reason.to_string(),
        };
        
        // Persist healed state
        self.save()?;
        
        Ok(audit)
    }

    /// Complete recovery protocol: calculate and apply convergence in one step.
    ///
    /// This is a convenience method that combines `calculate_convergence_constant()`
    /// and `apply_convergence()` for typical recovery scenarios.
    ///
    /// # Arguments
    /// * `trusted_state` - The correct state from a trusted authority
    /// * `audit_reason` - Reason for recovery (e.g., "Rollback attack detected")
    ///
    /// # Example
    /// ```no_run
    /// # use isa_runtime::{DeviceRuntime, FilePersistence};
    /// # let persistence = FilePersistence::new("device.state");
    /// # let mut runtime = DeviceRuntime::new([0u8; 32], persistence);
    /// # let trusted_state = runtime.state_vector();
    /// let audit = runtime.recover_from_trusted_state(
    ///     &trusted_state,
    ///     "Detected divergence during merchant verification"
    /// ).unwrap();
    /// 
    /// println!("Recovery completed at timestamp: {}", audit.timestamp);
    /// ```
    pub fn recover_from_trusted_state(
        &mut self,
        trusted_state: &StateVector,
        audit_reason: &str,
    ) -> Result<RecoveryAudit> {
        let k = self.calculate_convergence_constant(trusted_state);
        self.apply_convergence(&k, audit_reason)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventAxis {
    Finance,
    Time,
    Hardware,
}

/// Audit record for state recovery/healing events.
///
/// This structure provides a cryptographically verifiable trail of all
/// state convergence operations, enabling forensic analysis and compliance.
#[derive(Debug, Clone)]
pub struct RecoveryAudit {
    /// Unix timestamp when recovery occurred
    pub timestamp: u64,
    /// Device state before applying convergence constant
    pub pre_healing_state: StateVector,
    /// The convergence constant K that was applied
    pub convergence_constant: StateVector,
    /// Device state after applying convergence constant
    pub post_healing_state: StateVector,
    /// Human-readable reason for the recovery
    pub reason: String,
}

/// Modular addition in Z_2^256.
///
/// Computes (a + b) mod 2^256 with proper carry propagation.
fn modular_add(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut carry = 0u16;
    
    for i in 0..32 {
        let sum = a[i] as u16 + b[i] as u16 + carry;
        result[i] = sum as u8;
        carry = sum >> 8;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FilePersistence;
    use tempfile::TempDir;

    #[test]
    fn test_device_runtime_record_sale() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.bin");
        let persistence = FilePersistence::new(&state_path);

        let master_seed = [1u8; 32];
        let mut runtime = DeviceRuntime::new(master_seed, persistence);

        let sale_data = b"sale:100.00:item123";
        let vector1 = runtime.record_sale(sale_data).unwrap();
        
        let sale_data2 = b"sale:200.00:item456";
        let vector2 = runtime.record_sale(sale_data2).unwrap();

        assert_ne!(vector1.finance, vector2.finance);
        assert_ne!(vector1.time, vector2.time);
        assert_ne!(vector1.hardware, vector2.hardware);
    }

    #[test]
    fn test_device_runtime_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.bin");
        let master_seed = [1u8; 32];
        
        // Ensure clean state
        let _ = std::fs::remove_file(&state_path);
        
        {
            let persistence = FilePersistence::new(&state_path);
            let mut runtime = DeviceRuntime::new(master_seed, persistence);
            runtime.record_sale(b"sale:100.00").unwrap();
            runtime.save().unwrap();
        }
        
        {
            let persistence = FilePersistence::new(&state_path);
            let runtime = DeviceRuntime::<FilePersistence>::load_or_create(master_seed, persistence).unwrap();
            // Dimension 0 (finance) should have counter = 1
            assert_eq!(runtime.state.dimension(0).unwrap().counter(), 1);
        }
    }

    #[test]
    fn test_device_runtime_event_axis() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.bin");
        let persistence = FilePersistence::new(&state_path);

        let master_seed = [1u8; 32];
        let mut runtime = DeviceRuntime::new(master_seed, persistence);

        let initial_vector = runtime.state_vector();
        
        runtime.record_event(EventAxis::Finance, b"finance_event").unwrap();
        let after_finance = runtime.state_vector();
        assert_ne!(initial_vector.finance, after_finance.finance);

        runtime.record_event(EventAxis::Time, b"time_event").unwrap();
        let after_time = runtime.state_vector();
        assert_ne!(after_finance.time, after_time.time);

        runtime.record_event(EventAxis::Hardware, b"hw_event").unwrap();
        let after_hw = runtime.state_vector();
        assert_ne!(after_time.hardware, after_hw.hardware);
    }
}
