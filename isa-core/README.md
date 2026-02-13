# isa-core

Pure cryptographic implementation of the Multi-Axis Integral State Accumulator.

## Features

- `no_std` compatible (with `default-features = false`)
- Zero unsafe code
- Deterministic across all platforms
- Constant-time operations where applicable

## Usage

```rust
use isa_core::{AxisAccumulator, MultiAxisState};

// Single axis
let mut axis = AxisAccumulator::new([0u8; 32]);
axis.accumulate(b"event", b"entropy", 1000);
let state = axis.state();

// Multi-axis
let mut state = MultiAxisState::from_master_seed([0u8; 32]);
let vector = state.state_vector();
```

## Cryptographic Guarantees

1. **Irreversibility**: Cannot compute previous state from current state
2. **Avalanche**: Single-bit input change affects ~50% of output bits
3. **Determinism**: Same inputs always produce same outputs
4. **Collision Resistance**: Inherited from BLAKE3

## no_std Support

```toml
[dependencies]
isa-core = { version = "0.1", default-features = false }
```
