//! Tests for the recovery protocol (Equation 8 from the paper).
//!
//! Validates that state convergence works correctly and that
//! divergence is eliminated after applying the convergence constant K.

use isa_runtime::{DeviceRuntime, FilePersistence};
use isa_core::{CircularDistance, StateVector};
use tempfile::TempDir;

#[test]
fn test_convergence_constant_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("device.state");
    let persistence = FilePersistence::new(&state_path);
    
    let master_seed = [0x42; 32];
    let runtime = DeviceRuntime::new(master_seed, persistence);
    
    // Create a "trusted" state (simulating authority)
    let trusted_state = runtime.state_vector();
    
    // Calculate K = (S_honest - S_drifted) mod 2^256
    let k = runtime.calculate_convergence_constant(&trusted_state);
    
    // K should be zero since states are identical
    assert_eq!(k.finance, [0u8; 32]);
    assert_eq!(k.time, [0u8; 32]);
    assert_eq!(k.hardware, [0u8; 32]);
}

#[test]
fn test_state_restoration_theorem() {
    // Theorem 2: For any drifted state S'_n, there exists K such that
    // D(S_n, (S'_n + K)) = 0
    
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("device.state");
    let persistence = FilePersistence::new(&state_path);
    
    let master_seed = [0x42; 32];
    let mut runtime = DeviceRuntime::new(master_seed, persistence);
    
    // Record some events to create the "honest" trajectory
    runtime.record_sale(b"sale:100.00").unwrap();
    runtime.record_sale(b"sale:200.00").unwrap();
    let honest_state = runtime.state_vector();
    
    // Simulate drift by manually altering dimension 0 (finance)
    let counter = runtime.state.dimension(0).unwrap().counter();
    *runtime.state.dimension_mut(0).unwrap() = isa_core::DimensionAccumulator::from_state(
        [0x13; 32], // Drifted state
        counter,
    );
    let drifted_state = runtime.state_vector();
    
    // Verify divergence exists
    let divergence_before = CircularDistance::compute(
        &honest_state.finance,
        &drifted_state.finance,
    );
    assert_ne!(divergence_before, [0u8; 32]);
    
    // Apply recovery protocol
    let audit = runtime.recover_from_trusted_state(
        &honest_state,
        "Test: Theorem 2 validation"
    ).unwrap();
    
    let restored_state = runtime.state_vector();
    
    // Verify D(S_honest, S_restored) = 0
    let divergence_after = CircularDistance::compute(
        &honest_state.finance,
        &restored_state.finance,
    );
    assert_eq!(divergence_after, [0u8; 32], "Divergence should be zero after recovery");
    
    // Verify audit trail
    assert!(audit.timestamp > 0);
    assert_eq!(audit.reason, "Test: Theorem 2 validation");
    assert_eq!(audit.post_healing_state.finance, honest_state.finance);
}

#[test]
fn test_recovery_audit_trail() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("device.state");
    let persistence = FilePersistence::new(&state_path);
    
    let master_seed = [0x42; 32];
    let mut runtime = DeviceRuntime::new(master_seed, persistence);
    
    // Create honest trajectory
    runtime.record_sale(b"sale:100.00").unwrap();
    let honest_state = runtime.state_vector();
    
    // Simulate rollback attack on dimension 0 (finance)
    let counter = runtime.state.dimension(0).unwrap().counter();
    *runtime.state.dimension_mut(0).unwrap() = isa_core::DimensionAccumulator::from_state(
        [0xFF; 32],
        counter,
    );
    
    let pre_healing = runtime.state_vector();
    
    // Recover
    let audit = runtime.recover_from_trusted_state(
        &honest_state,
        "Rollback attack detected by merchant"
    ).unwrap();
    
    // Verify audit contains all necessary information
    assert_eq!(audit.pre_healing_state.finance, pre_healing.finance);
    assert_eq!(audit.post_healing_state.finance, honest_state.finance);
    assert_ne!(audit.convergence_constant.finance, [0u8; 32]);
    assert_eq!(audit.reason, "Rollback attack detected by merchant");
}

#[test]
fn test_multi_axis_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("device.state");
    let persistence = FilePersistence::new(&state_path);
    
    let master_seed = [0x42; 32];
    let mut runtime = DeviceRuntime::new(master_seed, persistence);
    
    // Create honest state across all axes
    runtime.record_sale(b"sale:100.00").unwrap();
    let honest_state = runtime.state_vector();
    
    // Tamper with all three dimensions
    *runtime.state.dimension_mut(0).unwrap() = isa_core::DimensionAccumulator::from_state([0x11; 32], 1);
    *runtime.state.dimension_mut(1).unwrap() = isa_core::DimensionAccumulator::from_state([0x22; 32], 1);
    *runtime.state.dimension_mut(2).unwrap() = isa_core::DimensionAccumulator::from_state([0x33; 32], 1);
    
    // Calculate divergence on all axes
    let divergence = runtime.calculate_divergence(&honest_state);
    assert_ne!(divergence.finance, [0u8; 32]);
    assert_ne!(divergence.time, [0u8; 32]);
    assert_ne!(divergence.hardware, [0u8; 32]);
    
    // Recover all axes
    runtime.recover_from_trusted_state(
        &honest_state,
        "Multi-axis tampering detected"
    ).unwrap();
    
    let restored = runtime.state_vector();
    
    // Verify all axes restored
    assert_eq!(restored.finance, honest_state.finance);
    assert_eq!(restored.time, honest_state.time);
    assert_eq!(restored.hardware, honest_state.hardware);
}

#[test]
fn test_calculate_divergence() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("device.state");
    let persistence = FilePersistence::new(&state_path);
    
    let master_seed = [0x42; 32];
    let mut runtime = DeviceRuntime::new(master_seed, persistence);
    
    runtime.record_sale(b"sale:100.00").unwrap();
    let checkpoint = runtime.state_vector();
    
    // Continue recording events
    runtime.record_sale(b"sale:200.00").unwrap();
    runtime.record_sale(b"sale:300.00").unwrap();
    
    // Calculate divergence from checkpoint
    let divergence = runtime.calculate_divergence(&checkpoint);
    
    // Divergence should be non-zero since we've recorded more events
    assert_ne!(divergence.finance, [0u8; 32]);
}

#[test]
fn test_recovery_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("device.state");
    
    let master_seed = [0x42; 32];
    let honest_state: StateVector;
    
    // Create and recover in first session
    {
        let persistence = FilePersistence::new(&state_path);
        let mut runtime = DeviceRuntime::new(master_seed, persistence);
        
        runtime.record_sale(b"sale:100.00").unwrap();
        honest_state = runtime.state_vector();
        
        // Tamper with dimension 0 (finance)
        *runtime.state.dimension_mut(0).unwrap() = isa_core::DimensionAccumulator::from_state([0xFF; 32], 1);
        
        // Recover and save
        runtime.recover_from_trusted_state(&honest_state, "Test recovery").unwrap();
    }
    
    // Verify recovery persisted
    {
        let persistence = FilePersistence::new(&state_path);
        let runtime = DeviceRuntime::load_or_create(master_seed, persistence).unwrap();
        
        let loaded_state = runtime.state_vector();
        assert_eq!(loaded_state.finance, honest_state.finance);
    }
}
