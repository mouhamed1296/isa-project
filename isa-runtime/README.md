# isa-runtime

Platform-aware runtime for MA-ISA with entropy gathering, time sources, and persistence.

## Features

- Monotonic time source
- Hardware entropy via `getrandom`
- Atomic file persistence
- Cross-platform support (including WASM)

## Usage

```rust
use isa_runtime::{DeviceRuntime, FilePersistence};

let persistence = FilePersistence::new("./state.bin");
let mut runtime = DeviceRuntime::load_or_create(
    master_seed,
    persistence
)?;

// Record events
runtime.record_sale(b"sale:100.00")?;
runtime.save()?;
```

## Platform Support

- **Native**: Full support with filesystem persistence
- **WASM**: Uses `getrandom` with JS entropy source
- **Embedded**: Compatible with `no_std` environments (with custom persistence)
