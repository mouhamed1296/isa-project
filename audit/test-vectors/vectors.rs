//! Deterministic test vectors for MA-ISA.
//!
//! These vectors serve as canonical reference standards.
//! They MUST pass identically across all platforms.
//!
//! ## Rules
//!
//! - No randomness in tests
//! - Fixed seeds, events, entropy, and time deltas
//! - Expected values are frozen and treated as canonical
//! - Never regenerate unless the math intentionally changes

use isa_core::{AxisAccumulator, MultiAxisState, CircularDistance};

/// Test vector 001: Basic single-axis accumulation
#[test]
fn vector_001_basic_accumulation() {
    let seed = [0u8; 32];
    let event = b"sale:1000";
    let entropy = b"device:pos_dakar_01";
    let delta_t = 1u64;

    let mut axis = AxisAccumulator::new(seed);
    axis.accumulate(event, entropy, delta_t);

    let state = axis.state();
    
    // Canonical expected value (frozen)
    let expected = hex::decode("68c9a8830584b71046044df26986f3d531f4b71e274b37ef0c2cc83cf0e75b8b").unwrap();
    let expected_array: [u8; 32] = expected.try_into().unwrap();
    
    assert_eq!(state, expected_array, "Vector 001 failed: basic accumulation");
    assert_eq!(axis.counter(), 1);
}

/// Test vector 002: Sequential accumulation
#[test]
fn vector_002_sequential_accumulation() {
    let seed = [0u8; 32];
    let mut axis = AxisAccumulator::new(seed);

    // First accumulation
    axis.accumulate(b"event1", b"entropy1", 100);
    let state1 = axis.state();
    
    let expected1 = hex::decode("7b8da26af96e3364d905e49ac38255d43e1d95665886e2fcf72839c7c6fca35b").unwrap();
    let expected1_array: [u8; 32] = expected1.try_into().unwrap();
    assert_eq!(state1, expected1_array, "Vector 002 failed: first accumulation");

    // Second accumulation
    axis.accumulate(b"event2", b"entropy2", 200);
    let state2 = axis.state();
    
    let expected2 = hex::decode("e552703cc9872d124140448e99aa0e729f4dad97176cbd174af3637e1b5f8cc1").unwrap();
    let expected2_array: [u8; 32] = expected2.try_into().unwrap();
    assert_eq!(state2, expected2_array, "Vector 002 failed: second accumulation");
    
    assert_eq!(axis.counter(), 2);
}

/// Test vector 003: Multi-axis state from master seed
#[test]
fn vector_003_multi_axis_from_seed() {
    let master_seed = [1u8; 32];
    let state = MultiAxisState::from_master_seed(master_seed);
    
    let vector = state.state_vector();
    
    // Canonical expected values for each axis (frozen)
    let expected_finance = hex::decode("994445bab17833bddcf2f1b36baf657842b2e38650b9e1cac3306462efdc76da").unwrap();
    let expected_time = hex::decode("e406691993bcb7f7ac6f24399b6aeef2a149cbf3e7c98356bfd5bd07a8ec2869").unwrap();
    let expected_hardware = hex::decode("2fdabf911d7f4ac491142876e43ae1794f38accf963ceb0f320bee637c8da9e2").unwrap();
    
    let expected_finance_array: [u8; 32] = expected_finance.try_into().unwrap();
    let expected_time_array: [u8; 32] = expected_time.try_into().unwrap();
    let expected_hardware_array: [u8; 32] = expected_hardware.try_into().unwrap();
    
    assert_eq!(vector.finance, expected_finance_array, "Vector 003 failed: finance axis");
    assert_eq!(vector.time, expected_time_array, "Vector 003 failed: time axis");
    assert_eq!(vector.hardware, expected_hardware_array, "Vector 003 failed: hardware axis");
}

/// Test vector 004: Divergence calculation
#[test]
fn vector_004_divergence() {
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    
    a[0] = 10;
    b[0] = 5;
    
    let distance = CircularDistance::compute(&a, &b);
    
    // Expected: simple subtraction in first byte
    assert_eq!(distance[0], 5, "Vector 004 failed: simple divergence");
    for i in 1..32 {
        assert_eq!(distance[i], 0, "Vector 004 failed: remaining bytes should be zero");
    }
}

/// Test vector 005: Wraparound divergence
#[test]
fn vector_005_wraparound_divergence() {
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    
    a[0] = 5;
    b[0] = 10;
    
    let distance = CircularDistance::compute(&a, &b);
    
    // Expected: wraparound (256 - 5 = 251)
    assert_eq!(distance[0], 251, "Vector 005 failed: wraparound divergence");
}

/// Test vector 006: Zero divergence
#[test]
fn vector_006_zero_divergence() {
    let master_seed = [42u8; 32];
    let state1 = MultiAxisState::from_master_seed(master_seed);
    let state2 = MultiAxisState::from_master_seed(master_seed);
    
    let div = state1.divergence(&state2);
    
    assert_eq!(div.finance, [0u8; 32], "Vector 006 failed: finance divergence should be zero");
    assert_eq!(div.time, [0u8; 32], "Vector 006 failed: time divergence should be zero");
    assert_eq!(div.hardware, [0u8; 32], "Vector 006 failed: hardware divergence should be zero");
}

/// Test vector 007: Determinism across platforms
#[test]
fn vector_007_cross_platform_determinism() {
    // This test ensures the same inputs produce the same outputs
    // regardless of platform (x86, ARM, WASM, etc.)
    
    let seed = [0x42u8; 32];
    let event = b"cross_platform_test";
    let entropy = b"fixed_entropy_source";
    let delta_t = 12345u64;
    
    let mut axis = AxisAccumulator::new(seed);
    axis.accumulate(event, entropy, delta_t);
    
    let state = axis.state();
    
    // This value must be identical on all platforms
    let expected = hex::decode("88f78a1be16d288f74d9470df247de5f45bdc6bafd587062d0404625a43c0d23").unwrap();
    let expected_array: [u8; 32] = expected.try_into().unwrap();
    
    assert_eq!(state, expected_array, "Vector 007 failed: cross-platform determinism violated");
}

/// Test vector 008: Counter wrapping behavior
#[test]
fn vector_008_counter_wrapping() {
    let seed = [0u8; 32];
    let mut axis = AxisAccumulator::from_state(seed, u64::MAX);
    
    // This should wrap to 0
    axis.accumulate(b"wrap_test", b"entropy", 1);
    
    assert_eq!(axis.counter(), 0, "Vector 008 failed: counter should wrap to 0");
}

/// Test vector 009: Empty event and entropy
#[test]
fn vector_009_empty_inputs() {
    let seed = [0u8; 32];
    let mut axis = AxisAccumulator::new(seed);
    
    axis.accumulate(b"", b"", 0);
    let state = axis.state();
    
    // Even with empty inputs, state should change deterministically
    let expected = hex::decode("4cf05c8006ef81a3c7e27920dd5a8e103fc47941c32616d2278ca2f00dfde1ed").unwrap();
    let expected_array: [u8; 32] = expected.try_into().unwrap();
    
    assert_eq!(state, expected_array, "Vector 009 failed: empty inputs");
}

/// Test vector 010: Large delta_t
#[test]
fn vector_010_large_delta_t() {
    let seed = [0u8; 32];
    let mut axis = AxisAccumulator::new(seed);
    
    axis.accumulate(b"event", b"entropy", u64::MAX);
    let state = axis.state();
    
    // Large delta_t should be handled deterministically
    let expected = hex::decode("728cb5cbcfd0f9ad35722ef822f89f8928be9c4f95a96e46efb170e3eb6d8895").unwrap();
    let expected_array: [u8; 32] = expected.try_into().unwrap();
    
    assert_eq!(state, expected_array, "Vector 010 failed: large delta_t");
}
