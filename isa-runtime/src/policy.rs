//! Integrity dimension policies for threshold evaluation and state reconciliation.
//!
//! ## Conformance Classification
//!
//! **NORMATIVE (Partial)** - Threshold evaluation logic is required for conformance.
//! State reconciliation strategies are implementation-specific and informative only.
//!
//! This module defines per-dimension policies that control:
//! - Divergence thresholds (when to trigger reconciliation) - **NORMATIVE**
//! - State reconciliation strategies (how to restore integrity) - **INFORMATIVE**
//! - Monitoring and alerting rules - **INFORMATIVE**
//!
//! ## Terminology
//!
//! - "Reconciliation" replaces informal term "healing"
//! - "Safety-relevant dimension" replaces "critical dimension"

use isa_core::STATE_SIZE;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Recovery strategy for a dimension when divergence exceeds threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RecoveryStrategy {
    /// Immediately apply convergence constant to heal the dimension.
    ImmediateHeal,
    
    /// Log the divergence but continue operating (monitoring only).
    MonitorOnly,
    
    /// Quarantine the dimension and prevent further accumulation.
    Quarantine,
    
    /// Trigger a full system recovery protocol.
    FullRecovery,
    
    /// Use a custom recovery function (index into a registry).
    Custom(u32),
}

/// Policy for a single dimension.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionPolicy {
    /// Name/label for this dimension (for logging/debugging).
    pub name: String,
    
    /// Divergence threshold in bytes.
    /// If the first N bytes of divergence exceed this value, trigger recovery.
    pub threshold_bytes: usize,
    
    /// Maximum allowed divergence value (as u64 from first 8 bytes).
    pub max_divergence: u64,
    
    /// Recovery strategy to use when threshold is exceeded.
    pub recovery_strategy: RecoveryStrategy,
    
    /// Whether this dimension is critical (affects system-wide decisions).
    pub is_critical: bool,
    
    /// Weight/importance of this dimension (0.0 to 1.0).
    pub weight: f32,
    
    /// Whether accumulation is currently enabled for this dimension.
    pub enabled: bool,
}

impl DimensionPolicy {
    /// Create a new dimension policy with default settings.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            threshold_bytes: 8,
            max_divergence: u64::MAX / 2, // 50% of state space
            recovery_strategy: RecoveryStrategy::ImmediateHeal,
            is_critical: false,
            weight: 1.0,
            enabled: true,
        }
    }
    
    /// Set the divergence threshold.
    pub fn with_threshold(mut self, max_divergence: u64) -> Self {
        self.max_divergence = max_divergence;
        self
    }
    
    /// Set the recovery strategy.
    pub fn with_recovery(mut self, strategy: RecoveryStrategy) -> Self {
        self.recovery_strategy = strategy;
        self
    }
    
    /// Mark this dimension as critical.
    pub fn critical(mut self) -> Self {
        self.is_critical = true;
        self
    }
    
    /// Set the weight/importance of this dimension.
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }
    
    /// Check if a divergence value exceeds this policy's threshold.
    pub fn exceeds_threshold(&self, divergence: &[u8; STATE_SIZE]) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Convert first 8 bytes to u64 for comparison
        let div_value = u64::from_le_bytes([
            divergence[0], divergence[1], divergence[2], divergence[3],
            divergence[4], divergence[5], divergence[6], divergence[7],
        ]);
        
        div_value > self.max_divergence
    }
}

/// Policy set for all dimensions in an integrity state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PolicySet {
    policies: Vec<DimensionPolicy>,
}

impl PolicySet {
    /// Create a new empty policy set.
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }
    
    /// Add a policy for a dimension.
    pub fn add_policy(&mut self, policy: DimensionPolicy) {
        self.policies.push(policy);
    }
    
    /// Get the policy for a specific dimension index.
    pub fn get(&self, index: usize) -> Option<&DimensionPolicy> {
        self.policies.get(index)
    }
    
    /// Get a mutable reference to a policy.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut DimensionPolicy> {
        self.policies.get_mut(index)
    }
    
    /// Get the number of policies.
    pub fn len(&self) -> usize {
        self.policies.len()
    }
    
    /// Check if the policy set is empty.
    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }
    
    /// Evaluate all policies against divergence values.
    ///
    /// Returns a vector of (dimension_index, policy) pairs for dimensions
    /// that exceed their thresholds.
    pub fn evaluate(&self, divergences: &[[u8; STATE_SIZE]]) -> Vec<(usize, &DimensionPolicy)> {
        let mut violations = Vec::new();
        
        for (i, div) in divergences.iter().enumerate() {
            if let Some(policy) = self.get(i) {
                if policy.exceeds_threshold(div) {
                    violations.push((i, policy));
                }
            }
        }
        
        violations
    }
}

impl Default for PolicySet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_creation() {
        let policy = DimensionPolicy::new("test")
            .with_threshold(1000)
            .critical()
            .with_weight(0.8);
        
        assert_eq!(policy.name, "test");
        assert_eq!(policy.max_divergence, 1000);
        assert!(policy.is_critical);
        assert_eq!(policy.weight, 0.8);
    }
    
    #[test]
    fn test_threshold_check() {
        let policy = DimensionPolicy::new("test").with_threshold(100);
        
        let low_div = [50u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let high_div = [200u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        
        assert!(!policy.exceeds_threshold(&low_div));
        assert!(policy.exceeds_threshold(&high_div));
    }
    
    #[test]
    fn test_policy_set() {
        let mut policy_set = PolicySet::new();
        policy_set.add_policy(DimensionPolicy::new("dim0").with_threshold(100));
        policy_set.add_policy(DimensionPolicy::new("dim1").with_threshold(200));
        
        assert_eq!(policy_set.len(), 2);
        assert_eq!(policy_set.get(0).unwrap().name, "dim0");
    }
}
