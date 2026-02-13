# Security Audit Package

This directory contains all materials needed for a security audit of MA-ISA.

## Audit Scope

### In Scope
- **isa-core** crate (~2000 LOC)
  - Cryptographic primitives (BLAKE3, SHA-256)
  - State accumulation logic
  - Divergence calculation
  - Key derivation function (KDF)
  - Formal verification harnesses

### Out of Scope
- **isa-runtime** - Wrapper around isa-core (business logic)
- **isa-ffi** - C/WASM bindings (FFI layer only)
- **isa-merkle** - Merkle tree implementation (uses isa-core)
- Network security, persistence layer, language bindings

## What We're Looking For

### Critical Issues
1. **State Tampering** - Can state be modified without detection?
2. **Collision Attacks** - Can two different inputs produce same state?
3. **Replay Attacks** - Can old states be restored undetected?
4. **Cryptographic Weaknesses** - Flaws in hash usage or KDF?
5. **Formal Verification Gaps** - Are the Kani proofs sound?

### High Priority
1. Side-channel vulnerabilities (timing attacks)
2. Integer overflow/underflow issues
3. Divergence calculation correctness
4. Cross-platform determinism guarantees

### Medium Priority
1. Code quality and maintainability
2. Documentation accuracy
3. Test coverage gaps
4. Dependency security

## Materials Provided

### 1. Source Code
- `../isa-core/` - Complete source code
- `../isa-core/src/verify.rs` - Formal verification harnesses
- `../isa-core/tests/` - Test suite

### 2. Documentation
- `../ARCHITECTURE.md` - System design
- `../SECURITY_GUARANTEES.md` - Security properties
- `../isa-core/VERIFICATION.md` - Formal verification details
- `THREAT_MODEL.md` - Attack scenarios (this directory)

### 3. Test Artifacts
- `test-vectors/` - Deterministic test vectors (this directory)
- `../isa-core/tests/vectors.rs` - Cross-platform tests
- `dependencies.txt` - Full dependency tree (this directory)

### 4. Formal Proofs
- `formal-proofs/` - Kani verification results (this directory)
- 10 properties verified (see SECURITY_GUARANTEES.md)

## How to Build & Test

```bash
# Build isa-core
cd ../isa-core
cargo build --release

# Run tests
cargo test

# Run formal verification (requires Kani)
cargo kani

# Run benchmarks
cargo bench --bench benchmarks

# Check for unsafe code
cargo geiger
```

## Timeline & Budget

**Preferred Timeline:** Q2 2026 (flexible)
**Budget:** $10,000 - $20,000
**Duration:** 1-2 weeks

## Deliverables

1. **Security Assessment Report**
   - Executive summary
   - Detailed findings with severity ratings
   - Proof-of-concept code for vulnerabilities
   - Remediation recommendations

2. **Public Report** (preferred)
   - Sanitized version for public disclosure
   - Can be used in marketing materials

3. **Follow-up** (optional)
   - Re-audit after fixes
   - Ongoing security advisory

## Contact

**Project Maintainer:** [Your Name]
**Email:** [your-email]
**GitHub:** https://github.com/[username]/isa-project

## Questions for Auditors

1. Do you have experience with Rust cryptography?
2. Can you review formal verification approaches (Kani)?
3. What's your typical timeline for this scope?
4. Do you provide a public report option?
5. What's your process for critical findings?

## Audit Firms Contacted

- [ ] Trail of Bits
- [ ] NCC Group  
- [ ] Cure53
- [ ] [Add others as contacted]

## Notes

- This is a **pre-production audit** - no live systems deployed yet
- Focus on cryptographic correctness, not performance
- We value transparency - willing to publish findings
- Open to suggestions for additional testing/verification
