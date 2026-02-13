# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-03

### Added

#### isa-core (NORMATIVE)
- Initial implementation of `AxisAccumulator` with BLAKE3-based state mixing
- `IntegrityState<N>` for domain-agnostic multi-dimensional integrity monitoring
- **Dynamic dimensions** (`DynamicIntegrityState`) - runtime-configurable dimension count
- Circular distance divergence metric over 2^256 space
- Deterministic KDF using BLAKE3
- Version embedding in serialized state
- `no_std` compatibility
- Comprehensive test suite with deterministic test vectors (27/27 tests passing)

#### isa-runtime (MIXED)
- **Dimension policies** (`policy.rs` - NORMATIVE) - per-dimension thresholds and recovery strategies
- **Configuration system** (`config.rs` - NORMATIVE) - YAML/JSON/TOML loading + environment variables
- **Cross-dimension constraints** (`constraints.rs` - OPTIONAL) - MaxRatio, SumBelow, Conditional checks
- **Dimension hierarchies** (`hierarchy.rs` - OPTIONAL) - parent-child relationships and aggregation
- **Adaptive profiles** (`adaptive.rs` - EXPERIMENTAL) - ML-driven dimension importance learning
- `DeviceRuntime` for platform-aware state management
- `MonotonicClock` for time source
- `EntropySource` using getrandom
- `FilePersistence` with atomic writes
- WASM support via getrandom JS feature
- Event recording API (23/23 tests passing)

#### isa-ffi
- Complete C ABI with handle-based resource management
- WASM bindings via wasm-bindgen
- C header file (ma_isa.h)
- Null pointer validation on all FFI boundaries
- Error code enum for cross-language error handling

#### isa-cli
- Command-line interface with 6 commands
- State inspection and verification
- Record keeping and comparison

#### isa-merkle (OPTIONAL)
- Merkle tree batch verification
- Efficient verification of 1000s of devices
- BLAKE3-based Merkle trees (8/8 tests passing)

#### Configuration System
- **config-examples/** directory with YAML/JSON/TOML templates
- Environment variable support (`ISA_DIM*` prefix)
- Multi-language usage guide (Python, JavaScript, Go, Java, C#)
- Hot-reload capable configuration
- Kubernetes ConfigMap and Docker environment variable examples

#### Documentation
- **README.md** - Updated with domain-agnostic design and configuration examples
- **ENHANCEMENTS.md** - Complete documentation of 5 major features
- **CONFORMANCE.md** - ISO/IEC conformance specification
- **SECURITY.md** - Security policy with conformance levels and configuration security
- **ROADMAP.md** - Updated with completed v0.1.0 features
- **CONTRIBUTING.md** - Updated with conformance levels and docs organization
- Per-crate README files
- Configuration examples with multi-language guide

### Security
- **Zero unsafe code** in NORMATIVE modules (isa-core, policy.rs, config.rs)
- **Constant-time equality** checks using subtle crate
- **Memory zeroization** on drop for sensitive data
- **Version compatibility** checks on deserialization
- **Avalanche effect** verification in tests
- **Configuration validation** - Input validation for YAML/JSON/TOML
- **Environment variable security** - Secrets management recommendations
- **Conformance-based security** - Clear NORMATIVE/OPTIONAL/EXPERIMENTAL classification

### Changed
- **Domain-agnostic design** - Removed hardcoded finance/time/hardware axes
- **Terminology standardization** - ISO-compliant terms (dimension, reconciliation, etc.)
- Restructured `isa-core` to canonical layout
- Added module-level conformance documentation to all modules
- Frozen ABI surface with stability markers
- Documentation organization - moved internal docs to `docs/` folder (gitignored)

### Performance
- **Formal verification** using Kani model checker (10 properties verified)
- **SIMD hardware acceleration** - AVX2 (3-4x speedup), NEON (2-3x speedup)
- **Performance benchmarks** using Criterion.rs
- O(1) accumulation operations (~1-2 μs per call)
- ~140 bytes serialized state size

### Testing
- **58/58 tests passing** across all modules
- isa-core: 27/27 tests
- isa-runtime: 23/23 tests
- isa-merkle: 8/8 tests
- Cross-platform determinism verified (x86_64, ARM64, WASM)
- 10 deterministic test vectors

### ISO/IEC Conformance
- Module classification (NORMATIVE/OPTIONAL/EXPERIMENTAL)
- Conformance specification document
- RFC 2119 keyword usage (SHALL/SHOULD/MAY)
- Formal conformance statement
- Security policy aligned with conformance levels

## [Unreleased]

### Planned for v0.2.0 (April 2026)
- Security audit (Trail of Bits, NCC Group, or Kudelski)
- Academic paper publication (arXiv submission)
- Community security review
- Bug bounty program
- Web demo with WASM

### Planned for v0.3.0 (June 2026)
- Zero-Knowledge protocol (MA-ISA-ZK)
- Production-ready adaptive algorithms
- Context-aware threshold calculation

### Planned for v0.4.0 (September 2026)
- Post-quantum cryptography integration
- Hardware security module support (SGX, TrustZone)
- TEE integration

### Community Contributions Welcome
- **Configuration examples** - Domain-specific configs (IoT, finance, healthcare)
- **Language-specific examples** - Python, Go, Java config loading
- **Native bindings** - Python (PyO3), Go (cgo), Swift, Node.js (N-API)
- **Visualization tools** - Dimension hierarchy visualization
- **Additional constraint types** - Beyond MaxRatio/SumBelow
- **Production ML algorithms** - For adaptive profiles
