//! Domain-agnostic integrity dimension primitives.
//!
//! This module defines the core abstraction for multi-dimensional state accumulation
//! without any domain-specific semantics. Dimensions are identified by numeric indices.
//!
//! ## Design Principles
//!
//! - No domain semantics (finance, time, hardware, etc.)
//! - Dimensions are interchangeable and parameterized
//! - All operations are deterministic and platform-independent
//! - Cryptographic logic is unchanged from original AxisAccumulator

use crate::axis::AxisAccumulator;
use crate::STATE_SIZE;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single integrity dimension.
///
/// This is a thin wrapper around `AxisAccumulator` that removes domain semantics.
/// The underlying cryptographic accumulation logic is identical.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionAccumulator {
    inner: AxisAccumulator,
}

impl DimensionAccumulator {
    /// Create a new dimension accumulator from a seed.
    pub fn new(seed: [u8; STATE_SIZE]) -> Self {
        Self {
            inner: AxisAccumulator::new(seed),
        }
    }

    /// Accumulate an event into this dimension.
    ///
    /// This is mathematically identical to the original axis accumulation:
    /// S_n = (S_{n-1} + Φ(event, entropy, delta_t)) mod 2^256
    pub fn accumulate(&mut self, event: &[u8], entropy: &[u8], delta_t: u64) {
        self.inner.accumulate(event, entropy, delta_t);
    }

    /// Get the current state of this dimension.
    pub fn state(&self) -> [u8; STATE_SIZE] {
        self.inner.state()
    }

    /// Get the event counter for this dimension.
    pub fn counter(&self) -> u64 {
        self.inner.counter()
    }

    /// Create a dimension accumulator from an existing state and counter.
    ///
    /// Used for state restoration and recovery protocols.
    pub fn from_state(state: [u8; STATE_SIZE], counter: u64) -> Self {
        Self {
            inner: AxisAccumulator::from_state(state, counter),
        }
    }
}

impl core::fmt::Debug for DimensionAccumulator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DimensionAccumulator")
            .field("inner", &self.inner)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_accumulator_creation() {
        let seed = [1u8; 32];
        let dim = DimensionAccumulator::new(seed);
        assert_eq!(dim.counter(), 0);
    }

    #[test]
    fn test_dimension_accumulation() {
        let seed = [1u8; 32];
        let mut dim = DimensionAccumulator::new(seed);
        
        let initial_state = dim.state();
        dim.accumulate(b"event", &[0u8; 16], 1000);
        
        assert_ne!(dim.state(), initial_state);
        assert_eq!(dim.counter(), 1);
    }

    #[test]
    fn test_dimension_determinism() {
        let seed = [1u8; 32];
        let mut dim1 = DimensionAccumulator::new(seed);
        let mut dim2 = DimensionAccumulator::new(seed);
        
        dim1.accumulate(b"event", &[0u8; 16], 1000);
        dim2.accumulate(b"event", &[0u8; 16], 1000);
        
        assert_eq!(dim1.state(), dim2.state());
        assert_eq!(dim1.counter(), dim2.counter());
    }
}
