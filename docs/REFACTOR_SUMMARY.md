# MA-ISA Structural Refactor Summary

## Objective

Align the existing MA-ISA Rust project to a canonical 3-crate architecture without changing any cryptographic or mathematical logic.

## Changes Made

### ✅ STEP 1: Workspace Alignment

**File Restructuring in `isa-core`:**

| Old File | New File | Contains |
|----------|----------|----------|
| `src/accumulator.rs` | `src/axis.rs` | `AxisAccumulator` (single-axis primitive) |
| `src/state.rs` | `src/accumulator.rs` | `MultiAxisState` (multi-axis coordinator) |
| `src/divergence.rs` | `src/divergence.rs` | `CircularDistance` (unchanged) |
| `src/kdf.rs` | `src/kdf.rs` | KDF functions (unchanged) |
| `src/version.rs` | `src/version.rs` | Version struct (unchanged) |

**Module Updates:**
- Updated `isa-core/src/lib.rs` to export from new module structure
- Updated all internal imports to use `axis` module instead of `accumulator`
- Added module-level invariant documentation to all modules

### ✅ STEP 2: ABI Surface Frozen

**Marked as ABI-Stable in `isa-ffi/src/c_api.rs`:**

All C ABI functions and types now have explicit stability markers:

```rust
/// **ABI STABLE — DO NOT CHANGE WITHOUT MAJOR VERSION**
```

**Frozen ABI Functions:**
- `isa_runtime_new` - Create runtime with opaque handle
- `isa_runtime_load_or_create` - Load or create runtime
- `isa_runtime_free` - Free runtime handle
- `isa_record_sale` - Record sale event
- `isa_record_event` - Record event on specific axis
- `isa_save` - Persist state
- `isa_get_state_vector` - Get current state
- `isa_axis_new` - Create single axis
- `isa_axis_free` - Free axis
- `isa_axis_accumulate` - Accumulate into axis
- `isa_axis_get_state` - Get axis state
- `isa_state_new` - Create multi-axis state
- `isa_state_free` - Free state
- `isa_get_version` - Get library version

**Frozen ABI Types:**
- `StateVectorC` - `#[repr(C)]` struct with three 32-byte arrays
- `FfiError` - `#[repr(C)]` enum with fixed integer values

**ABI Guarantees:**
- All structs use `#[repr(C)]`
- All functions use `#[no_mangle]` and `extern "C"`
- Opaque handles (usize) for complex types
- No Rust-specific types cross the boundary
- No panics across FFI boundaries

### ✅ STEP 3: Deterministic Test Vectors

**Created `isa-core/tests/vectors.rs`:**

10 canonical test vectors that serve as reference standards:

1. **vector_001_basic_accumulation** - Single accumulation with fixed inputs
2. **vector_002_sequential_accumulation** - Two sequential accumulations
3. **vector_003_multi_axis_from_seed** - Multi-axis state derivation
4. **vector_004_divergence** - Simple divergence calculation
5. **vector_005_wraparound_divergence** - Wraparound in modular arithmetic
6. **vector_006_zero_divergence** - Identical states have zero divergence
7. **vector_007_cross_platform_determinism** - Cross-platform verification
8. **vector_008_counter_wrapping** - Counter overflow behavior
9. **vector_009_empty_inputs** - Empty event and entropy handling
10. **vector_010_large_delta_t** - Maximum delta_t value

**Test Vector Properties:**
- No randomness - all inputs are fixed
- Expected values are frozen hex strings
- Must pass identically across all platforms
- Serve as canonical reference for correctness

**Helper Tool:**
- Created `examples/generate_test_vectors.rs` to regenerate expected values if math intentionally changes

### ✅ STEP 4: Invariant Documentation

**Added module-level documentation to each crate:**

**`isa-core/src/lib.rs`:**
```rust
//! This crate must remain deterministic, platform-independent, and free of side effects.
//! Any use of time, randomness, IO, or system state is **FORBIDDEN**.
```

**`isa-runtime/src/lib.rs`:**
```rust
//! This crate is **ALLOWED** to:
//! - Use system time sources
//! - Generate hardware entropy
//! - Perform file I/O for persistence
//!
//! This crate **MUST NOT**:
//! - Implement cryptographic primitives (use isa-core)
//! - Expose language bindings (use isa-ffi)
```

**`isa-ffi/src/lib.rs`:**
```rust
//! This crate is **ALLOWED** to:
//! - Use unsafe code (isolated to FFI boundaries only)
//! - Expose C-compatible types and functions
//!
//! This crate **MUST NOT**:
//! - Implement cryptographic logic (use isa-core)
//! - Expose Rust-specific types across the ABI
//! - Panic across FFI boundaries
```

## What Was NOT Changed

### ❌ No Logic Changes

- **Zero changes** to cryptographic algorithms
- **Zero changes** to mathematical operations
- **Zero changes** to state transition logic
- **Zero changes** to KDF implementation
- **Zero changes** to divergence calculations

### ✅ Only Structural Changes

- File renaming
- Module path updates
- Import statement adjustments
- Documentation additions
- Test vector creation

## Verification

### Build Status
```
✅ cargo build --all: SUCCESS
✅ cargo test --all: 39 tests passed (29 existing + 10 new vectors)
```

### Test Results
- **isa-core**: 20 unit tests + 10 test vectors = 30 tests ✅
- **isa-runtime**: 9 tests ✅
- **isa-ffi**: 0 tests (FFI boundary only)

**Total: 39/39 tests passing**

## ABI Function Signatures

### Runtime Management
```c
usize isa_runtime_new(const uint8_t* seed, const char* path);
usize isa_runtime_load_or_create(const uint8_t* seed, const char* path);
isa_error_t isa_runtime_free(usize handle);
```

### Event Recording
```c
isa_error_t isa_record_sale(usize handle, const uint8_t* data, size_t len, isa_state_vector_t* out);
isa_error_t isa_record_event(usize handle, uint8_t axis, const uint8_t* data, size_t len, isa_state_vector_t* out);
isa_error_t isa_save(usize handle);
isa_error_t isa_get_state_vector(usize handle, isa_state_vector_t* out);
```

### Low-Level Axis API
```c
void* isa_axis_new(const uint8_t* seed);
void isa_axis_free(void* axis);
isa_error_t isa_axis_accumulate(void* axis, const uint8_t* event, size_t event_len, 
                                 const uint8_t* entropy, size_t entropy_len, uint64_t delta_t);
isa_error_t isa_axis_get_state(const void* axis, uint8_t* out);
```

### State Management
```c
void* isa_state_new(const uint8_t* seed);
void isa_state_free(void* state);
```

### Version Info
```c
isa_error_t isa_get_version(uint16_t* major, uint16_t* minor, uint16_t* patch);
```

## Risks and Assumptions

### Assumptions
1. **Test vectors are correct**: Expected values were generated by running the existing code
2. **No hidden state**: All state is captured in the serialized format
3. **Platform independence**: BLAKE3 and integer arithmetic are deterministic across platforms

### Risks
- **Breaking changes require major version**: Any ABI change requires v1.0.0 → v2.0.0
- **Test vectors are immutable**: Changing math requires regenerating all vectors
- **C header must stay in sync**: Manual updates to `include/ma_isa.h` required

## Migration Impact

### For Rust Users
- **Import changes**: `use isa_core::AxisAccumulator` unchanged (re-exported)
- **API unchanged**: All public APIs remain identical
- **No code changes required**

### For C Users
- **No changes**: C ABI is frozen and unchanged
- **Header unchanged**: `include/ma_isa.h` remains compatible

### For WASM Users
- **No changes**: WASM bindings unchanged

## Final Structure

```
ma-isa/
├── Cargo.toml                    # Workspace root
├── isa-core/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs               # Module exports + invariants
│   │   ├── axis.rs              # AxisAccumulator (single-axis)
│   │   ├── accumulator.rs       # MultiAxisState (multi-axis)
│   │   ├── divergence.rs        # CircularDistance
│   │   ├── kdf.rs               # BLAKE3 KDF
│   │   └── version.rs           # Version struct
│   ├── tests/
│   │   └── vectors.rs           # 10 canonical test vectors
│   └── examples/
│       └── generate_test_vectors.rs
├── isa-runtime/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Runtime exports + invariants
│       ├── device.rs            # DeviceRuntime
│       ├── entropy.rs           # EntropySource
│       ├── time.rs              # MonotonicClock
│       └── persistence.rs       # FilePersistence
└── isa-ffi/
    ├── Cargo.toml
    ├── include/
    │   └── ma_isa.h             # C header (frozen ABI)
    └── src/
        ├── lib.rs               # FFI exports + invariants
        ├── c_api.rs             # C ABI (frozen, marked stable)
        ├── error.rs             # FfiError enum (frozen)
        └── wasm.rs              # WASM bindings
```

## Confirmation

✅ **No cryptographic logic was changed**  
✅ **No mathematical operations were modified**  
✅ **Only structural refactoring was performed**  
✅ **All tests pass (39/39)**  
✅ **ABI surface is frozen and documented**  
✅ **Deterministic test vectors are in place**  
✅ **Invariants are documented in each crate**  

---

**Refactor Status: COMPLETE**  
**Build Status: ✅ PASSING**  
**Test Status: ✅ 39/39 PASSING**  
**ABI Status: ✅ FROZEN**
