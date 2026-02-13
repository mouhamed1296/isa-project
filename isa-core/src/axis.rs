//! Single integrity dimension accumulator primitive.
//!
//! ## Conformance Classification
//!
//! **NORMATIVE** - This module defines required behavior for MA-ISA conformance.
//!
//! This module contains the core `AxisAccumulator` type, which maintains
//! an irreversible cryptographic state that evolves with each event.
//!
//! ## Normative Requirements
//!
//! - All state transitions SHALL be deterministic
//! - Implementations SHALL NOT use randomness, time, or IO operations
//! - State mixing SHALL use only pure cryptographic functions
//! - Counter increments SHALL be wrapping (no overflow panics)

use crate::kdf::mix_state;
use crate::STATE_SIZE;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisAccumulator {
    state: [u8; STATE_SIZE],
    #[zeroize(skip)]
    counter: u64,
}

impl AxisAccumulator {
    pub fn new(seed: [u8; 32]) -> Self {
        Self {
            state: seed,
            counter: 0,
        }
    }

    pub fn accumulate(&mut self, event: &[u8], entropy: &[u8], delta_t: u64) {
        self.state = mix_state(&self.state, event, entropy, delta_t);
        self.counter = self.counter.wrapping_add(1);
    }

    pub fn state(&self) -> [u8; 32] {
        self.state
    }

    pub fn counter(&self) -> u64 {
        self.counter
    }

    pub fn from_state(state: [u8; 32], counter: u64) -> Self {
        Self { state, counter }
    }
}

impl core::fmt::Debug for AxisAccumulator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AxisAccumulator")
            .field("state", &"[REDACTED]")
            .field("counter", &self.counter)
            .finish()
    }
}

impl PartialEq for AxisAccumulator {
    fn eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.state.ct_eq(&other.state).into() && self.counter == other.counter
    }
}

impl Eq for AxisAccumulator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator_deterministic() {
        let seed = [0u8; 32];
        let mut acc1 = AxisAccumulator::new(seed);
        let mut acc2 = AxisAccumulator::new(seed);

        acc1.accumulate(b"event1", b"entropy1", 100);
        acc2.accumulate(b"event1", b"entropy1", 100);

        assert_eq!(acc1.state(), acc2.state());
        assert_eq!(acc1.counter(), acc2.counter());
    }

    #[test]
    fn test_accumulator_sequence() {
        let seed = [0u8; 32];
        let mut acc = AxisAccumulator::new(seed);

        let state0 = acc.state();
        acc.accumulate(b"event1", b"entropy1", 100);
        let state1 = acc.state();
        acc.accumulate(b"event2", b"entropy2", 200);
        let state2 = acc.state();

        assert_ne!(state0, state1);
        assert_ne!(state1, state2);
        assert_ne!(state0, state2);
        assert_eq!(acc.counter(), 2);
    }

    #[test]
    fn test_accumulator_irreversible() {
        let seed = [0u8; 32];
        let mut acc = AxisAccumulator::new(seed);

        acc.accumulate(b"event1", b"entropy1", 100);
        let state1 = acc.state();

        acc.accumulate(b"event2", b"entropy2", 200);
        let state2 = acc.state();

        assert_ne!(state1, state2);
    }
}
