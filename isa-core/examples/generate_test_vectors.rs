use isa_core::{AxisAccumulator, MultiAxisState};

fn main() {
    // Vector 001
    {
        let seed = [0u8; 32];
        let event = b"sale:1000";
        let entropy = b"device:pos_dakar_01";
        let delta_t = 1u64;
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(event, entropy, delta_t);
        println!("Vector 001: {}", hex::encode(axis.state()));
    }

    // Vector 002 - first accumulation
    {
        let seed = [0u8; 32];
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(b"event1", b"entropy1", 100);
        println!("Vector 002 first:  {}", hex::encode(axis.state()));
        axis.accumulate(b"event2", b"entropy2", 200);
        println!("Vector 002 second: {}", hex::encode(axis.state()));
    }

    // Vector 003 - all three axes
    {
        let master_seed = [1u8; 32];
        let state = MultiAxisState::from_master_seed(master_seed);
        let vector = state.state_vector();
        println!("Vector 003 dimension 0:  {}", hex::encode(vector.values[0]));
        println!("Vector 003 dimension 1:  {}", hex::encode(vector.values[1]));
        println!("Vector 003 dimension 2:  {}", hex::encode(vector.values[2]));
    }

    // Vector 007
    {
        let seed = [0x42u8; 32];
        let event = b"cross_platform_test";
        let entropy = b"fixed_entropy_source";
        let delta_t = 12345u64;
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(event, entropy, delta_t);
        println!("Vector 007: {}", hex::encode(axis.state()));
    }

    // Vector 009
    {
        let seed = [0u8; 32];
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(b"", b"", 0);
        println!("Vector 009: {}", hex::encode(axis.state()));
    }

    // Vector 010
    {
        let seed = [0u8; 32];
        let mut axis = AxisAccumulator::new(seed);
        axis.accumulate(b"event", b"entropy", u64::MAX);
        println!("Vector 010: {}", hex::encode(axis.state()));
    }
}
