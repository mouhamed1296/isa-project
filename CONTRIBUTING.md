# Contributing to MA-ISA

**Version:** 0.1.0  
**Last Updated:** February 3, 2026

Thank you for your interest in contributing to the Multi-Axis Integral State Accumulator project!

## 📚 Documentation Organization

### Public Documentation (Published to GitHub)
These files are in the project root and are part of the public repository:
- `README.md` - Main project documentation
- `CONTRIBUTING.md` - This file
- `CHANGELOG.md` - Version history
- `SECURITY.md` - Security policy and threat model
- `ROADMAP.md` - Project roadmap
- `CONFORMANCE.md` - ISO/IEC conformance specification
- `ENHANCEMENTS.md` - Feature documentation

### Internal Documentation (NOT Published)
The `docs/` folder contains internal context documentation that is gitignored:
- Architecture notes
- Implementation summaries
- Research papers
- Working documents

**Note:** Do not commit files to `docs/` - they are for local development only.

## 🎯 Priority Contributions

### Configuration Examples

We welcome configuration examples for different use cases and languages!

#### Domain-Specific Configurations
- **IoT devices**: Low-dimension, resource-constrained configs
- **Financial systems**: High-security, multi-dimension configs
- **Healthcare**: HIPAA-compliant configurations
- **Government**: Compliance-focused configurations

#### Language-Specific Examples
- **Python**: YAML loading with `pyyaml`
- **JavaScript**: JSON/YAML with environment variable fallback
- **Go**: TOML with `viper` configuration
- **Java**: Properties files with Spring Boot

Add examples to `config-examples/` with clear README documentation.

### Language Bindings

We welcome language bindings for MA-ISA! The C ABI is frozen and stable.

**Note:** For many languages, you can use the configuration system without native bindings:
- Load YAML/JSON/TOML config files in your language
- Set environment variables (`ISA_DIM*`)
- Call MA-ISA via subprocess or FFI

#### Native Bindings (Advanced)
- **Python**: PyO3 for native bindings
- **Go**: cgo to call C ABI (`isa-ffi/include/ma_isa.h`)
- **Swift**: Swift C interop for iOS/macOS
- **Node.js**: WASM (existing) or N-API

### Enhancements & Features

#### OPTIONAL Module Improvements
- **Constraints**: Additional constraint types beyond MaxRatio/SumBelow
- **Hierarchies**: Visualization tools for dimension trees
- **Configuration**: Additional config formats (HCL, Dhall, etc.)

#### EXPERIMENTAL Module Contributions
- **Adaptive Profiles**: Production-ready ML algorithms
- **Context-aware thresholds**: Dynamic threshold calculation
- **Anomaly detection**: Pattern recognition in divergence

**Note:** Experimental modules must remain clearly marked as non-production.

#### Performance Enhancements
- **SIMD**: Already implemented (AVX2, NEON) - optimizations welcome
- **Batch operations**: Merkle tree integration (see `isa-merkle`)
- **Parallel processing**: Multi-threaded divergence calculation

#### Formal Verification
- **Current**: 10 properties verified with Kani
- **Opportunities**: Additional properties, Coq/Isabelle proofs
- **Target**: NORMATIVE modules only (`isa-core`, `policy.rs`, `config.rs`)

## 📋 Contribution Guidelines

### Code Standards

1. **Understand conformance levels**
   - **NORMATIVE**: Changes require security review and ISO consideration
   - **OPTIONAL**: Changes allowed with tests and documentation
   - **EXPERIMENTAL**: Clearly mark as non-production

2. **No logic changes to NORMATIVE modules** without discussion
   - `isa-core/*` - All core cryptographic primitives
   - `isa-runtime/policy.rs` - Threshold evaluation
   - `isa-runtime/config.rs` - Configuration loading
   - Changes require security review

3. **Follow existing patterns**
   - Match code style in each crate
   - Use existing error handling patterns
   - Maintain domain-agnostic design

4. **Add tests**
   - Unit tests for new functionality
   - Integration tests for bindings
   - Configuration validation tests
   - Benchmark new performance-critical code

5. **Document everything**
   - Public APIs need doc comments
   - Complex logic needs inline comments
   - Update README files
   - Add conformance classification to new modules

### Testing Requirements

- All tests must pass: `cargo test --workspace` (currently 58/58)
- Benchmarks must not regress: `cargo bench --bench benchmarks`
- New code needs >80% test coverage
- Test vectors must remain deterministic
- Configuration examples must be validated
- Cross-platform compatibility (x86_64, ARM64, WASM)

### Pull Request Process

1. **Fork and create a branch**
   ```bash
   git checkout -b feature/python-bindings
   ```

2. **Make your changes**
   - Follow code standards above
   - Add tests
   - Update documentation

3. **Verify everything works**
   ```bash
   cargo test --all
   cargo bench --bench benchmarks
   cargo clippy --all-targets
   ```

4. **Submit PR**
   - Clear description of changes
   - Link to any related issues
   - Include benchmark results if relevant

## 🏗️ Project Structure

```
ma-isa/
├── isa-core/           # NORMATIVE: Pure cryptographic primitives
│   ├── axis.rs         # Single dimension accumulator
│   ├── dynamic.rs      # Runtime-configurable dimensions
│   └── integrity_state.rs  # Multi-dimensional state
├── isa-runtime/        # MIXED: Platform-aware runtime
│   ├── policy.rs       # NORMATIVE: Threshold evaluation
│   ├── config.rs       # NORMATIVE: Configuration loading
│   ├── constraints.rs  # OPTIONAL: Cross-dimension constraints
│   ├── hierarchy.rs    # OPTIONAL: Dimension hierarchies
│   └── adaptive.rs     # EXPERIMENTAL: ML-based optimization
├── isa-ffi/            # C ABI + WASM bindings
├── isa-cli/            # Command-line interface
├── isa-merkle/         # Optional Merkle tree proofs
├── config-examples/    # Configuration templates (YAML/JSON/TOML)
├── docs/               # Internal documentation (gitignored)
└── [your-crate]/       # New contributions go here
```

### Adding a New Crate

1. Create directory: `mkdir isa-[language]`
2. Add to workspace: Edit root `Cargo.toml`
3. Depend on `isa-core` and/or `isa-runtime`
4. Add README with usage examples
5. Add tests

## 🔐 Security

### Reporting Vulnerabilities

**DO NOT** open public issues for security vulnerabilities.

Email: security@[your-domain] (or create SECURITY.md)

### Security Review Required

Changes to NORMATIVE modules require security review:
- **isa-core**: All files (cryptographic primitives)
- **isa-runtime/policy.rs**: Threshold evaluation logic
- **isa-runtime/config.rs**: Configuration loading and validation

Changes to OPTIONAL modules require testing but not security review:
- **isa-runtime/constraints.rs**
- **isa-runtime/hierarchy.rs**

Changes to EXPERIMENTAL modules must maintain clear warnings:
- **isa-runtime/adaptive.rs** (NOT for production use)

## 📚 Resources

### Public Documentation
- [README.md](README.md) - Project overview and quick start
- [ENHANCEMENTS.md](ENHANCEMENTS.md) - Feature documentation
- [CONFORMANCE.md](CONFORMANCE.md) - ISO/IEC conformance specification
- [SECURITY.md](SECURITY.md) - Security policy and threat model
- [ROADMAP.md](ROADMAP.md) - Project roadmap
- [config-examples/README.md](config-examples/README.md) - Configuration guide

### Internal Documentation (docs/ folder - gitignored)
- Architecture notes
- Implementation summaries
- Research papers
- Theory-to-code mapping

### C ABI Reference
- Header: `isa-ffi/include/ma_isa.h`
- Implementation: `isa-ffi/src/c_api.rs`
- All functions marked `ABI STABLE`

### Benchmarking
- Run: `cargo bench --bench benchmarks`
- Add new benchmarks to: `isa-core/benches/benchmarks.rs`

## 💬 Communication

### Questions?
- Open a discussion on GitHub
- Check existing issues first
- Be specific about your use case

### Feature Requests
- Open an issue with `[Feature]` prefix
- Describe the use case
- Explain why existing features don't work

## 🎉 Recognition

Contributors will be:
- Listed in CHANGELOG.md
- Credited in release notes
- Added to AUTHORS file (if created)

Thank you for contributing to MA-ISA!
