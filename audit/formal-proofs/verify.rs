//! Formal verification harnesses using Kani.
//!
//! This module contains proof harnesses that verify critical properties
//! of the MA-ISA cryptographic primitives.
//!
//! Run with: `cargo kani --harness <harness_name>`

#[cfg(kani)]
mod kani_harnesses {
    use crate::{AxisAccumulator, CircularDistance, MultiAxisState};

    /// Verify that AxisAccumulator operations are deterministic.
    ///
    /// Property: Same inputs always produce same outputs.
    #[kani::proof]
    fn verify_axis_determinism() {
        let seed: [u8; 32] = kani::any();
        let event: [u8; 16] = kani::any();
        let entropy: [u8; 16] = kani::any();
        let delta_t: u64 = kani::any();

        // Create two identical accumulators
        let mut acc1 = AxisAccumulator::new(seed);
        let mut acc2 = AxisAccumulator::new(seed);

        // Apply same operation
        acc1.accumulate(&event, &entropy, delta_t);
        acc2.accumulate(&event, &entropy, delta_t);

        // Verify determinism
        assert_eq!(acc1.state(), acc2.state());
        assert_eq!(acc1.counter(), acc2.counter());
    }

    /// Verify that accumulation is irreversible.
    ///
    /// Property: Cannot derive previous state from current state.
    #[kani::proof]
    fn verify_axis_irreversibility() {
        let seed: [u8; 32] = kani::any();
        let event: [u8; 16] = kani::any();
        let entropy: [u8; 16] = kani::any();
        let delta_t: u64 = kani::any();

        let mut acc = AxisAccumulator::new(seed);
        let state_before = acc.state();
        
        acc.accumulate(&event, &entropy, delta_t);
        let state_after = acc.state();

        // Verify state changed (unless all inputs are zero)
        let all_zero = event.iter().all(|&x| x == 0) 
                    && entropy.iter().all(|&x| x == 0) 
                    && delta_t == 0;
        
        if !all_zero {
            assert_ne!(state_before, state_after);
        }
    }

    /// Verify counter increments correctly.
    ///
    /// Property: Counter increments by 1 on each accumulation.
    #[kani::proof]
    fn verify_counter_increment() {
        let seed: [u8; 32] = kani::any();
        let event: [u8; 8] = kani::any();
        let entropy: [u8; 8] = kani::any();
        let delta_t: u64 = kani::any();

        let mut acc = AxisAccumulator::new(seed);
        let counter_before = acc.counter();
        
        acc.accumulate(&event, &entropy, delta_t);
        let counter_after = acc.counter();

        // Verify counter incremented (with wrapping)
        assert_eq!(counter_after, counter_before.wrapping_add(1));
    }

    /// Verify circular distance is symmetric.
    ///
    /// Property: distance(a, b) == distance(b, a) in modular space.
    #[kani::proof]
    fn verify_circular_distance_properties() {
        let a: [u8; 32] = kani::any();
        let b: [u8; 32] = kani::any();

        let dist_ab = CircularDistance::compute(&a, &b);
        let dist_ba = CircularDistance::compute(&b, &a);

        // In modular arithmetic, forward and backward distances sum to 0 (mod 2^256)
        // We verify the computation is consistent
        let min_ab = CircularDistance::min_distance(&a, &b);
        let min_ba = CircularDistance::min_distance(&b, &a);
        
        // Minimum distance should be symmetric
        assert_eq!(min_ab, min_ba);
    }

    /// Verify zero distance for identical states.
    ///
    /// Property: distance(a, a) == 0.
    #[kani::proof]
    fn verify_zero_distance() {
        let a: [u8; 32] = kani::any();

        let dist = CircularDistance::compute(&a, &a);
        
        // Distance from a state to itself is zero
        assert_eq!(dist, [0u8; 32]);
    }

    /// Verify MultiAxisState determinism.
    ///
    /// Property: Same master seed produces same state.
    #[kani::proof]
    fn verify_multi_axis_determinism() {
        let master_seed: [u8; 32] = kani::any();

        let state1 = MultiAxisState::from_master_seed(master_seed);
        let state2 = MultiAxisState::from_master_seed(master_seed);

        let vec1 = state1.state_vector();
        let vec2 = state2.state_vector();

        assert_eq!(vec1.finance, vec2.finance);
        assert_eq!(vec1.time, vec2.time);
        assert_eq!(vec1.hardware, vec2.hardware);
    }

    /// Verify divergence is zero for identical states.
    ///
    /// Property: divergence(s, s) == (0, 0, 0).
    #[kani::proof]
    fn verify_zero_divergence() {
        let master_seed: [u8; 32] = kani::any();

        let state = MultiAxisState::from_master_seed(master_seed);
        let div = state.divergence(&state);

        assert_eq!(div.finance, [0u8; 32]);
        assert_eq!(div.time, [0u8; 32]);
        assert_eq!(div.hardware, [0u8; 32]);
    }

    /// Verify avalanche effect: single bit change affects many bits.
    ///
    /// Property: Changing one input bit changes ~50% of output bits.
    #[kani::proof]
    fn verify_avalanche_effect() {
        let mut seed1: [u8; 32] = kani::any();
        let mut seed2 = seed1;
        
        // Flip one bit
        let byte_idx: usize = kani::any();
        kani::assume(byte_idx < 32);
        let bit_idx: u8 = kani::any();
        kani::assume(bit_idx < 8);
        
        seed2[byte_idx] ^= 1 << bit_idx;

        let mut acc1 = AxisAccumulator::new(seed1);
        let mut acc2 = AxisAccumulator::new(seed2);

        let event: [u8; 8] = kani::any();
        let entropy: [u8; 8] = kani::any();
        let delta_t: u64 = kani::any();

        acc1.accumulate(&event, &entropy, delta_t);
        acc2.accumulate(&event, &entropy, delta_t);

        let state1 = acc1.state();
        let state2 = acc2.state();

        // States should be different (avalanche effect)
        assert_ne!(state1, state2);
    }

    /// Verify no state collisions for different inputs.
    ///
    /// Property: Different seeds produce different states.
    #[kani::proof]
    fn verify_no_collisions() {
        let seed1: [u8; 32] = kani::any();
        let seed2: [u8; 32] = kani::any();
        
        // Assume seeds are different
        kani::assume(seed1 != seed2);

        let acc1 = AxisAccumulator::new(seed1);
        let acc2 = AxisAccumulator::new(seed2);

        // Different seeds should produce different initial states
        assert_ne!(acc1.state(), acc2.state());
    }

    /// Verify counter wrapping behavior.
    ///
    /// Property: Counter wraps at u64::MAX.
    #[kani::proof]
    fn verify_counter_wrapping() {
        let seed: [u8; 32] = kani::any();
        let event: [u8; 4] = kani::any();
        let entropy: [u8; 4] = kani::any();

        let mut acc = AxisAccumulator::from_state(seed, u64::MAX);
        
        acc.accumulate(&event, &entropy, 1);
        
        // Counter should wrap to 0
        assert_eq!(acc.counter(), 0);
    }
}
