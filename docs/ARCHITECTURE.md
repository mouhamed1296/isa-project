# MA-ISA Architecture

## Design Philosophy

MA-ISA follows strict architectural principles to ensure auditability, portability, and correctness.

## Crate Hierarchy

```
┌─────────────────────────────────────┐
│           isa-ffi                   │
│   (C ABI + WASM Bindings)          │
│   - Unsafe code isolated here       │
│   - No business logic               │
└──────────────┬──────────────────────┘
               │
               ├──────────────────────┐
               │                      │
┌──────────────▼──────────┐  ┌────────▼──────────┐
│     isa-runtime         │  │    isa-core       │
│  (Platform Runtime)     │  │  (Pure Crypto)    │
│  - Entropy gathering    │◄─┤  - no_std         │
│  - Time sources         │  │  - No unsafe      │
│  - Persistence          │  │  - Deterministic  │
└─────────────────────────┘  └───────────────────┘
```

## Layer Responsibilities

### isa-core (Pure Cryptography)

**Purpose**: Canonical implementation of MA-ISA mathematics

**Constraints**:
- `no_std` compatible
- Zero unsafe code
- No I/O, no clocks, no randomness (unless injected)
- Deterministic across all platforms

**Components**:
- `AxisAccumulator`: Single-axis state accumulation
- `MultiAxisState`: Three-axis coordinator
- `CircularDistance`: Divergence metric over 2^256
- `Kdf`: BLAKE3-based key derivation
- `Version`: Semantic versioning with compatibility checks

**Dependencies**: Only cryptographic primitives (blake3, sha2, subtle)

### isa-runtime (Platform Bridge)

**Purpose**: Bridge pure math to real-world devices

**Responsibilities**:
- Monotonic time source (`MonotonicClock`)
- Hardware entropy gathering (`EntropySource`)
- Persistent storage (`FilePersistence`)
- High-level device API (`DeviceRuntime`)

**Platform Support**:
- Native: Full filesystem and entropy support
- WASM: JS-based entropy, IndexedDB persistence
- Embedded: Custom persistence implementations

**Dependencies**: `isa-core` + platform libraries (getrandom, std::fs)

### isa-ffi (Language Bindings)

**Purpose**: Expose MA-ISA to other languages

**Targets**:
- C ABI: `cdylib` for native languages
- WASM: `wasm-bindgen` for JavaScript

**Safety**:
- All unsafe code isolated here
- Null pointer checks on all boundaries
- Handle-based resource management
- No cryptographic logic (delegates to core)

## Data Flow

### Initialization
```
Master Seed (32 bytes)
    │
    ├─► KDF("finance-axis") ──► Finance AxisAccumulator
    ├─► KDF("time-axis")    ──► Time AxisAccumulator
    └─► KDF("hardware-axis")──► Hardware AxisAccumulator
```

### Event Accumulation
```
Event Data + Entropy + ΔT
    │
    ▼
BLAKE3-KDF(current_state, event, entropy, delta_t)
    │
    ▼
New State (irreversible)
```

### Persistence
```
MultiAxisState
    │
    ├─► Add Version Header
    │
    ├─► bincode::serialize
    │
    ├─► Write to temp file
    │
    └─► Atomic rename
```

## State Format

### In-Memory
```rust
struct MultiAxisState {
    finance: AxisAccumulator,   // 32 bytes state + 8 bytes counter
    time: AxisAccumulator,       // 32 bytes state + 8 bytes counter
    hardware: AxisAccumulator,   // 32 bytes state + 8 bytes counter
    version: Version,            // 6 bytes (major, minor, patch)
}
```

### Serialized (bincode)
```
[Version Header: 6 bytes]
[Finance State: 32 bytes]
[Finance Counter: 8 bytes]
[Time State: 32 bytes]
[Time Counter: 8 bytes]
[Hardware State: 32 bytes]
[Hardware Counter: 8 bytes]
```

## Cryptographic Primitives

### BLAKE3 KDF

**Context**: `"MA-ISA-KDF-v1"`

**Inputs**:
- Context string (axis identifier)
- Current state (32 bytes)
- Event data (variable length)
- Entropy (variable length)
- Time delta (8 bytes, little-endian)

**Output**: 32 bytes

**Properties**:
- Deterministic
- Avalanche effect (1-bit change → ~128 bits change)
- Collision resistant (128-bit security)

### Circular Distance

**Purpose**: Measure divergence in modular 2^256 space

**Algorithm**:
```
distance(a, b) = min(|a - b|, 2^256 - |a - b|)
```

**Implementation**: Multi-precision integer subtraction with borrow

## Versioning Strategy

### Semantic Versioning

- **MAJOR**: Breaking changes to state format or cryptographic primitives
- **MINOR**: New axes, backward-compatible features
- **PATCH**: Bug fixes, no state format changes

### Compatibility Rules

```rust
fn is_compatible(v1: Version, v2: Version) -> bool {
    v1.major == v2.major
}
```

### Migration Path

1. Deserialize with version check
2. If incompatible, reject or migrate
3. Always serialize with current version

## Security Boundaries

### Trusted Computing Base (TCB)

**Minimal TCB**:
- `isa-core` (pure Rust, no unsafe)
- Cryptographic dependencies (blake3, subtle)
- Rust standard library (for std builds)

**Outside TCB**:
- `isa-runtime` (platform-specific, may fail)
- `isa-ffi` (unsafe code, input validation)
- Application code

### Threat Model

**Assumptions**:
- Master seed is securely generated and stored
- Entropy source is unpredictable
- Time source is monotonic (non-decreasing)

**Guarantees**:
- State cannot be forged without master seed
- State transitions are irreversible
- Divergence detection works even with clock skew

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| `accumulate()` | O(1) | Single BLAKE3 hash |
| `state()` | O(1) | Memory copy |
| `divergence()` | O(1) | 256-bit subtraction |
| `serialize()` | O(1) | Fixed-size structure |

### Space Complexity

| Component | Size | Notes |
|-----------|------|-------|
| `AxisAccumulator` | 40 bytes | 32 state + 8 counter |
| `MultiAxisState` | 126 bytes | 3 axes + version |
| Serialized | ~140 bytes | With bincode overhead |

### Benchmarks (Typical)

- Accumulation: ~1-2 μs
- Serialization: ~5-10 μs
- Deserialization: ~5-10 μs

## Extension Points

### Custom Axes

Add new axes by modifying `MultiAxisState`:

```rust
pub struct MultiAxisState {
    pub finance: AxisAccumulator,
    pub time: AxisAccumulator,
    pub hardware: AxisAccumulator,
    pub custom: AxisAccumulator,  // New axis
}
```

**Note**: This is a MINOR version bump.

### Custom Persistence

Implement the `Persistence` trait:

```rust
pub trait Persistence {
    fn save(&self, state: &MultiAxisState) -> Result<()>;
    fn load(&self) -> Result<MultiAxisState>;
    fn exists(&self) -> bool;
}
```

### Custom Entropy

Inject custom entropy into `accumulate()`:

```rust
let custom_entropy = my_entropy_source();
accumulator.accumulate(event, &custom_entropy, delta_t);
```

## Testing Strategy

### Unit Tests
- Each module has deterministic test vectors
- Avalanche effect verification
- Edge cases (zero inputs, max values)

### Integration Tests
- Cross-platform reproducibility
- Serialization round-trips
- Version compatibility

### Property Tests
- Commutativity (where applicable)
- Associativity (where applicable)
- Idempotence (where applicable)

### Fuzzing
- Input validation
- Serialization/deserialization
- State transitions

## Future Considerations

### Potential Enhancements
- Hardware acceleration (SIMD, GPU)
- Merkle tree integration for batch verification
- Zero-knowledge proofs for privacy-preserving verification
- Threshold signatures for multi-party control

### Research Directions
- Formal verification of core primitives
- Post-quantum cryptography migration path
- Homomorphic properties for encrypted computation
