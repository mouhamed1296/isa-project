//! Dynamic dimension support for runtime-configurable integrity states.
//!
//! This module provides `DynamicIntegrityState` which allows the number of dimensions
//! to be determined at runtime rather than compile-time.

use crate::dimension::DimensionAccumulator;
use crate::integrity_state::DimensionId;
use crate::kdf::Kdf;
use crate::version::Version;
use crate::STATE_SIZE;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Dynamic integrity state with runtime-configurable dimension count.
///
/// Unlike `IntegrityState<N>`, this type stores dimensions in a `Vec`,
/// allowing the dimension count to be determined at runtime.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DynamicIntegrityState {
    dimensions: Vec<DimensionAccumulator>,
    #[zeroize(skip)]
    version: Version,
}

impl DynamicIntegrityState {
    /// Create a new dynamic integrity state with the specified number of dimensions.
    pub fn new(dimension_count: usize, master_seed: [u8; STATE_SIZE]) -> Self {
        let mut dimensions = Vec::with_capacity(dimension_count);
        
        for i in 0..dimension_count {
            let dimension_id = DimensionId::from_index(i);
            let seed = Kdf::derive_key(&dimension_id.to_kdf_label(), &[&master_seed]);
            dimensions.push(DimensionAccumulator::new(seed));
        }
        
        Self {
            dimensions,
            version: Version::current(),
        }
    }
    
    /// Get the number of dimensions in this state.
    pub fn dimension_count(&self) -> usize {
        self.dimensions.len()
    }
    
    /// Get a reference to a specific dimension by index.
    pub fn dimension(&self, index: usize) -> Option<&DimensionAccumulator> {
        self.dimensions.get(index)
    }
    
    /// Get a mutable reference to a specific dimension by index.
    pub fn dimension_mut(&mut self, index: usize) -> Option<&mut DimensionAccumulator> {
        self.dimensions.get_mut(index)
    }
    
    /// Get the version of this state.
    pub fn version(&self) -> Version {
        self.version
    }
    
    /// Extract all dimension states as a vector.
    pub fn state_vector(&self) -> Vec<[u8; STATE_SIZE]> {
        self.dimensions.iter().map(|dim| dim.state()).collect()
    }
    
    /// Calculate divergence between this state and another.
    ///
    /// Returns None if the states have different dimension counts.
    pub fn divergence(&self, other: &Self) -> Option<Vec<[u8; STATE_SIZE]>> {
        if self.dimension_count() != other.dimension_count() {
            return None;
        }
        
        use crate::divergence::CircularDistance;
        
        let mut divergences = Vec::with_capacity(self.dimension_count());
        for i in 0..self.dimension_count() {
            divergences.push(CircularDistance::min_distance(
                &self.dimensions[i].state(),
                &other.dimensions[i].state(),
            ));
        }
        
        Some(divergences)
    }
    
    /// Add a new dimension to this state.
    ///
    /// The new dimension is initialized with a seed derived from the master seed
    /// and the new dimension index.
    pub fn add_dimension(&mut self, master_seed: [u8; STATE_SIZE]) {
        let new_index = self.dimensions.len();
        let dimension_id = DimensionId::from_index(new_index);
        let seed = Kdf::derive_key(&dimension_id.to_kdf_label(), &[&master_seed]);
        self.dimensions.push(DimensionAccumulator::new(seed));
    }
    
    /// Remove the last dimension from this state.
    ///
    /// Returns None if the state has no dimensions.
    pub fn remove_dimension(&mut self) -> Option<DimensionAccumulator> {
        self.dimensions.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dynamic_creation() {
        let master_seed = [1u8; 32];
        let state = DynamicIntegrityState::new(5, master_seed);
        assert_eq!(state.dimension_count(), 5);
    }
    
    #[test]
    fn test_add_remove_dimensions() {
        let master_seed = [1u8; 32];
        let mut state = DynamicIntegrityState::new(3, master_seed);
        assert_eq!(state.dimension_count(), 3);
        
        state.add_dimension(master_seed);
        assert_eq!(state.dimension_count(), 4);
        
        let removed = state.remove_dimension();
        assert!(removed.is_some());
        assert_eq!(state.dimension_count(), 3);
    }
    
    #[test]
    fn test_divergence_same_count() {
        let master_seed1 = [1u8; 32];
        let master_seed2 = [2u8; 32];
        
        let state1 = DynamicIntegrityState::new(3, master_seed1);
        let state2 = DynamicIntegrityState::new(3, master_seed2);
        
        let div = state1.divergence(&state2);
        assert!(div.is_some());
        assert_eq!(div.unwrap().len(), 3);
    }
    
    #[test]
    fn test_divergence_different_count() {
        let master_seed = [1u8; 32];
        
        let state1 = DynamicIntegrityState::new(3, master_seed);
        let state2 = DynamicIntegrityState::new(5, master_seed);
        
        let div = state1.divergence(&state2);
        assert!(div.is_none());
    }
}
