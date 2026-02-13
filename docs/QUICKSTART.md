# MA-ISA Quick Start Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
isa-core = { path = "path/to/isa-core", features = ["serde"] }
isa-runtime = { path = "path/to/isa-runtime" }
```

## Basic Usage

### 1. Single Axis Accumulator

```rust
use isa_core::AxisAccumulator;

let mut axis = AxisAccumulator::new([0u8; 32]);
axis.accumulate(b"event_data", b"entropy", 1000);
let state = axis.state();
```

### 2. Multi-Axis State

```rust
use isa_core::MultiAxisState;

let master_seed = [1u8; 32]; // Use secure random in production
let state = MultiAxisState::from_master_seed(master_seed);
let vector = state.state_vector();

println!("Finance: {:?}", vector.finance);
println!("Time: {:?}", vector.time);
println!("Hardware: {:?}", vector.hardware);
```

### 3. Device Runtime

```rust
use isa_runtime::{DeviceRuntime, FilePersistence};

let persistence = FilePersistence::new("./state.bin");
let mut runtime = DeviceRuntime::new(master_seed, persistence);

// Record a sale
let sale_data = b"sale:100.00:item123";
let vector = runtime.record_sale(sale_data)?;

// Persist state
runtime.save()?;
```

### 4. Load Existing State

```rust
let persistence = FilePersistence::new("./state.bin");
let runtime = DeviceRuntime::load_or_create(master_seed, persistence)?;
```

### 5. Calculate Divergence

```rust
let state1 = MultiAxisState::from_master_seed([1u8; 32]);
let state2 = MultiAxisState::from_master_seed([2u8; 32]);
let divergence = state1.divergence(&state2);
```

## C API Usage

```c
#include "ma_isa.h"

uint8_t seed[32] = {0};
isa_runtime_handle_t runtime = isa_runtime_new(seed, "./state.bin");

uint8_t sale[] = "sale:100.00:item123";
isa_state_vector_t vector;
isa_error_t err = isa_record_sale(runtime, sale, sizeof(sale), &vector);

if (err == ISA_SUCCESS) {
    isa_save(runtime);
}

isa_runtime_free(runtime);
```

## Building

```bash
# Build all crates
cargo build --release

# Run tests
cargo test --all

# Build C library
cd isa-ffi
cargo build --release
# Library at: target/release/libisa_ffi.{so,dylib,dll}

# Build WASM
cd isa-ffi
wasm-pack build --target web
```

## Security Best Practices

1. **Generate secure seeds**:
```rust
use getrandom::getrandom;
let mut seed = [0u8; 32];
getrandom(&mut seed).expect("RNG failure");
```

2. **Use different seeds per device**
3. **Protect seeds with HSM when available**
4. **Encrypt state at rest**
5. **Monitor divergence for anomalies**

## Testing

```bash
# Run all tests
cargo test --all

# Test specific crate
cargo test -p isa-core

# Test with verbose output
cargo test --all -- --nocapture
```

## Platform-Specific Notes

### Embedded (no_std)
```toml
[dependencies]
isa-core = { version = "0.1", default-features = false }
```

### WASM
```bash
cd isa-ffi
wasm-pack build --target web
# Output in pkg/
```

### Mobile (iOS/Android)
Use C bindings via FFI bridge.

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) for design details
- Review [SECURITY.md](SECURITY.md) for security considerations
- Check [README.md](README.md) for comprehensive documentation
