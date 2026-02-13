//! Backward compatibility layer for domain-specific API.
//!
//! This module provides type aliases and helper functions to maintain
//! backward compatibility with the original domain-specific API while
//! the core implementation is now domain-agnostic.
//!
//! ## Migration Path
//!
//! Old code using `MultiAxisState` can continue to work via type aliases.
//! New code should use `IntegrityState<N>` directly with dimension indices.

use crate::integrity_state::{IntegrityState, DimensionVector, DivergenceVector, IntegrityStateError, DimensionId};
use crate::dimension::DimensionAccumulator;
use crate::STATE_SIZE;

/// Backward-compatible alias for 3-dimensional integrity state.
///
/// This maps to the original MA-ISA model with three axes.
/// Dimension indices:
/// - 0: First dimension (was "finance")
/// - 1: Second dimension (was "time")  
/// - 2: Third dimension (was "hardware")
pub type MultiAxisState = IntegrityState<3>;

/// Backward-compatible state vector for 3 dimensions.
///
/// Provides named field access for compatibility while internally
/// using the dimension-indexed model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateVector {
    pub finance: [u8; STATE_SIZE],
    pub time: [u8; STATE_SIZE],
    pub hardware: [u8; STATE_SIZE],
}

impl From<DimensionVector<3>> for StateVector {
    fn from(vec: DimensionVector<3>) -> Self {
        Self {
            finance: vec.values[0],
            time: vec.values[1],
            hardware: vec.values[2],
        }
    }
}

impl From<StateVector> for DimensionVector<3> {
    fn from(vec: StateVector) -> Self {
        Self {
            values: [vec.finance, vec.time, vec.hardware],
        }
    }
}

/// Backward-compatible divergence metric for 3 dimensions.
#[derive(Debug, Clone, Copy)]
pub struct DivergenceMetric {
    pub finance: [u8; STATE_SIZE],
    pub time: [u8; STATE_SIZE],
    pub hardware: [u8; STATE_SIZE],
}

impl From<DivergenceVector<3>> for DivergenceMetric {
    fn from(vec: DivergenceVector<3>) -> Self {
        Self {
            finance: vec.values[0],
            time: vec.values[1],
            hardware: vec.values[2],
        }
    }
}

/// Backward-compatible error type.
pub type StateError = IntegrityStateError;

/// Extension trait to add domain-specific accessors to IntegrityState<3>.
///
/// This allows old code to continue using `.finance`, `.time`, `.hardware`
/// while the underlying implementation uses dimension indices.
pub trait MultiAxisStateExt {
    fn finance(&self) -> &DimensionAccumulator;
    fn time(&self) -> &DimensionAccumulator;
    fn hardware(&self) -> &DimensionAccumulator;
    fn finance_mut(&mut self) -> &mut DimensionAccumulator;
    fn time_mut(&mut self) -> &mut DimensionAccumulator;
    fn hardware_mut(&mut self) -> &mut DimensionAccumulator;
    fn state_vector_compat(&self) -> StateVector;
    fn divergence_compat(&self, other: &Self) -> DivergenceMetric;
}

impl MultiAxisStateExt for IntegrityState<3> {
    fn finance(&self) -> &DimensionAccumulator {
        self.dimension(0).expect("dimension 0 always exists")
    }

    fn time(&self) -> &DimensionAccumulator {
        self.dimension(1).expect("dimension 1 always exists")
    }

    fn hardware(&self) -> &DimensionAccumulator {
        self.dimension(2).expect("dimension 2 always exists")
    }

    fn finance_mut(&mut self) -> &mut DimensionAccumulator {
        self.dimension_mut(0).expect("dimension 0 always exists")
    }

    fn time_mut(&mut self) -> &mut DimensionAccumulator {
        self.dimension_mut(1).expect("dimension 1 always exists")
    }

    fn hardware_mut(&mut self) -> &mut DimensionAccumulator {
        self.dimension_mut(2).expect("dimension 2 always exists")
    }

    fn state_vector_compat(&self) -> StateVector {
        self.state_vector().into()
    }

    fn divergence_compat(&self, other: &Self) -> DivergenceMetric {
        self.divergence(other).into()
    }
}

/// Standard dimension IDs for the 3-axis MA-ISA configuration.
///
/// These provide stable, well-known dimension identifiers for the
/// original three-axis model without encoding semantic meaning.
pub mod standard_dimensions {
    use super::DimensionId;

    /// Dimension 0 identifier (was "finance-axis")
    pub fn dimension_0() -> DimensionId {
        DimensionId::from_index(0)
    }
    
    /// Dimension 1 identifier (was "time-axis")
    pub fn dimension_1() -> DimensionId {
        DimensionId::from_index(1)
    }
    
    /// Dimension 2 identifier (was "hardware-axis")
    pub fn dimension_2() -> DimensionId {
        DimensionId::from_index(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_vector_conversion() {
        let dim_vec = DimensionVector {
            values: [[1u8; 32], [2u8; 32], [3u8; 32]],
        };
        
        let state_vec: StateVector = dim_vec.into();
        assert_eq!(state_vec.finance, [1u8; 32]);
        assert_eq!(state_vec.time, [2u8; 32]);
        assert_eq!(state_vec.hardware, [3u8; 32]);
        
        let dim_vec2: DimensionVector<3> = state_vec.into();
        assert_eq!(dim_vec2.values, dim_vec.values);
    }

    #[test]
    fn test_multi_axis_state_ext() {
        let master_seed = [1u8; 32];
        let state: MultiAxisState = IntegrityState::from_master_seed(master_seed);
        
        // Test accessor methods
        assert_eq!(state.finance().counter(), 0);
        assert_eq!(state.time().counter(), 0);
        assert_eq!(state.hardware().counter(), 0);
        
        // Test state vector compatibility
        let vec = state.state_vector_compat();
        assert_eq!(vec.finance, state.finance().state());
        assert_eq!(vec.time, state.time().state());
        assert_eq!(vec.hardware, state.hardware().state());
    }

    #[test]
    fn test_divergence_compatibility() {
        let seed1 = [1u8; 32];
        let seed2 = [2u8; 32];
        let state1: MultiAxisState = IntegrityState::from_master_seed(seed1);
        let state2: MultiAxisState = IntegrityState::from_master_seed(seed2);
        
        let div = state1.divergence_compat(&state2);
        assert_ne!(div.finance, [0u8; 32]);
        assert_ne!(div.time, [0u8; 32]);
        assert_ne!(div.hardware, [0u8; 32]);
    }
}
