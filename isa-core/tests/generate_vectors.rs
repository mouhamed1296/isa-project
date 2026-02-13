//! Helper to generate canonical test vector values.
//! Run this once to get the expected hex values, then freeze them in vectors.rs

use isa_core::{AxisAccumulator, MultiAxisState};

fn main() {
    println!("=== Generating Test Vector Expected Values ===\n");

    // Vector 001
    {
        let seed = [0u8; 32];
        let event = b"sale:1000";
        let entropy = b"device:pos_dakar_01";
        let delta_t = 1u64;

        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(event, entropy, delta_t);
        let state = axis.state();
        
        println!("Vector 001 (basic_accumulation):");
        println!("  {}", hex::encode(state));
        println!();
    }

    // Vector 002 - first accumulation
    {
        let seed = [0u8; 32];
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(b"event1", b"entropy1", 100);
        let state1 = axis.state();
        
        println!("Vector 002 (sequential_accumulation - first):");
        println!("  {}", hex::encode(state1));
        
        axis.accumulate(b"event2", b"entropy2", 200);
        let state2 = axis.state();
        
        println!("Vector 002 (sequential_accumulation - second):");
        println!("  {}", hex::encode(state2));
        println!();
    }

    // Vector 003
    {
        let master_seed = [1u8; 32];
        let state = MultiAxisState::from_master_seed(master_seed);
        let vector = state.state_vector();
        
        println!("Vector 003 (multi_axis_from_seed):");
        println!("  dimension 0:  {}", hex::encode(vector.values[0]));
        println!("  dimension 1:  {}", hex::encode(vector.values[1]));
        println!("  dimension 2:  {}", hex::encode(vector.values[2]));
        println!();
    }

    // Vector 007
    {
        let seed = [0x42u8; 32];
        let event = b"cross_platform_test";
        let entropy = b"fixed_entropy_source";
        let delta_t = 12345u64;
        
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(event, entropy, delta_t);
        let state = axis.state();
        
        println!("Vector 007 (cross_platform_determinism):");
        println!("  {}", hex::encode(state));
        println!();
    }

    // Vector 009
    {
        let seed = [0u8; 32];
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(b"", b"", 0);
        let state = axis.state();
        
        println!("Vector 009 (empty_inputs):");
        println!("  {}", hex::encode(state));
        println!();
    }

    // Vector 010
    {
        let seed = [0u8; 32];
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(b"event", b"entropy", u64::MAX);
        let state = axis.state();
        
        println!("Vector 010 (large_delta_t):");
        println!("  {}", hex::encode(state));
        println!();
    }
}
