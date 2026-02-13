# Security Policy

**Version:** 0.1.0  
**Last Updated:** 2026-02-03  
**Status:** ISO/IEC Conformance Draft

## Cryptographic Guarantees

MA-ISA provides the following security properties:

### State Integrity (NORMATIVE)
- **Irreversibility**: State transitions are one-way functions
- **Avalanche Effect**: Single-bit mutations propagate across entire state
- **Collision Resistance**: Inherited from BLAKE3 (128-bit security)
- **Determinism**: Same inputs produce identical outputs across all platforms
- **Version Safety**: Embedded versioning prevents state format confusion

### Implementation Safety (NORMATIVE)
- **No Unsafe Code**: Core cryptographic primitives (`isa-core`) use only safe Rust
- **Constant-Time Operations**: Equality checks use `subtle::ConstantTimeEq`
- **Memory Zeroization**: Sensitive data is zeroized on drop via `zeroize` crate
- **Formal Verification**: 10 properties verified with Kani model checker
- **Test Coverage**: 58/58 tests passing across x86_64, ARM64, WASM

## Conformance Levels and Security

### NORMATIVE Modules (Required for Conformance)
**Security-Critical:**
- `isa-core/*` - All core cryptographic primitives
- `isa-runtime/policy.rs` - Threshold evaluation logic
- `isa-runtime/config.rs` - Configuration loading

These modules undergo strict security review and formal verification.

### OPTIONAL Modules (Conformance-Optional)
**Security-Relevant:**
- `isa-runtime/constraints.rs` - Cross-dimension constraints
- `isa-runtime/hierarchy.rs` - Dimension hierarchies

These modules are audited but not required for basic security guarantees.

### EXPERIMENTAL Modules (Non-Normative)
**⚠️ NOT for Production:**
- `isa-runtime/adaptive.rs` - ML-based optimization

**WARNING**: Experimental modules introduce non-determinism and SHALL NOT be used in:
- Safety-critical systems
- Regulatory compliance applications
- Formally verified systems
- High-security deployments

## Threat Model

### In Scope
✅ State forgery attempts  
✅ Replay attacks  
✅ Time manipulation  
✅ Entropy prediction  
✅ Side-channel attacks on comparison operations  
✅ Configuration injection attacks  
✅ Policy bypass attempts  

### Out of Scope
❌ Physical device compromise  
❌ Supply chain attacks  
❌ Compromised master seeds  
❌ Timing attacks on non-cryptographic operations  
❌ Malicious configuration files (validate externally)  
❌ ML model poisoning (adaptive module is experimental)  

## Best Practices

### Seed Generation
```rust
use getrandom::getrandom;

let mut master_seed = [0u8; 32];
getrandom(&mut master_seed).expect("RNG failure");

// For domain-agnostic usage
use isa_core::IntegrityState;
let state = IntegrityState::<5>::from_master_seed(master_seed);
```

### Seed Storage
- Use hardware security modules (HSM) when available
- Never hardcode seeds in source code
- Rotate seeds according to your security policy
- Use different seeds per device/deployment

### State Protection
- Encrypt serialized state at rest
- Use authenticated encryption for network transmission
- Validate version compatibility before deserialization
- Monitor divergence metrics for anomalies
- Validate configuration files before loading (YAML/JSON/TOML)
- Use environment variables for sensitive thresholds in production
- Never commit configuration files with production secrets to version control

## Reporting Vulnerabilities

**DO NOT** open public issues for security vulnerabilities.

Please report security issues via GitHub Security Advisories or contact the maintainers directly.

Include:
- **Module affected**: Specify if NORMATIVE/OPTIONAL/EXPERIMENTAL
- **Conformance level**: Does it affect conformance requirements?
- **Description**: Clear explanation of the vulnerability
- **Steps to reproduce**: Minimal test case
- **Potential impact**: Severity assessment
- **Suggested fix**: If available
- **Affected versions**: Which releases are vulnerable

We will:
1. Respond within 48 hours
2. Provide a timeline for fixes
3. Issue CVE if applicable
4. Release security patch as PATCH version
5. Update CHANGELOG.md with security advisory

## Configuration Security

### Configuration File Validation
When using external configuration (YAML/JSON/TOML):

```rust
use isa_runtime::config::IsaConfig;

// Validate configuration before use
let config = IsaConfig::from_file("policies.yaml")?;
if config.dimensions.is_empty() {
    return Err("Invalid configuration: no dimensions defined");
}

// Check for reasonable thresholds
for dim in &config.dimensions {
    if dim.threshold == 0 || dim.threshold > u64::MAX / 2 {
        return Err("Invalid threshold value");
    }
}
```

### Environment Variable Security
- Use `ISA_DIM*` prefix for all MA-ISA environment variables
- Validate all environment variable inputs
- Never log environment variable values
- Use secrets management systems (Vault, AWS Secrets Manager) in production

### Deployment Recommendations
1. **Development**: Use YAML/JSON config files (gitignored)
2. **Staging**: Use environment variables with validation
3. **Production**: Use secrets management + environment variables
4. **Kubernetes**: Use ConfigMaps for non-sensitive, Secrets for thresholds

## Audit Status

This is a reference implementation. For production use in high-security environments, we recommend:

1. Independent security audit of NORMATIVE modules
2. Formal verification of core primitives (10 properties already verified)
3. Penetration testing of configuration loading
4. Side-channel analysis of constant-time operations
5. Configuration validation testing

## Versioning and Updates

- Security patches: Released immediately as PATCH version
- Breaking security fixes: Released as MAJOR version with migration guide
- All security updates are documented in CHANGELOG.md

## Cryptographic Dependencies

| Crate | Version | Purpose | Security Level |
|-------|---------|---------|----------------|
| blake3 | 1.5+ | KDF and state mixing | 128-bit |
| sha2 | 0.10+ | (Optional) Alternative hash | 256-bit |
| subtle | 2.5+ | Constant-time operations | N/A |
| getrandom | 0.2+ | Entropy source | Platform-dependent |

All dependencies are well-audited and widely used in production systems.

## Compliance

MA-ISA is designed to be compatible with:
- **ISO/IEC Standards**: Conformance specification available in `CONFORMANCE.md`
- **FIPS 140-2**: When using approved primitives (BLAKE3, SHA-3)
- **Common Criteria EAL4+**: Core modules are deterministic and auditable
- **PCI DSS**: Suitable for payment terminal applications
- **GDPR**: No PII stored in state (domain-agnostic design)
- **SOC 2**: Audit trail and integrity monitoring capabilities

Actual compliance requires deployment-specific validation and certification.

## Module-Specific Security Notes

### isa-core (NORMATIVE)
- **Security Level**: Critical
- **Audit Status**: Kani-verified (10 properties)
- **Dependencies**: Minimal (blake3, subtle, zeroize)
- **no_std Compatible**: Yes (embedded-safe)

### isa-runtime (MIXED)
- **Security Level**: Varies by module
- **NORMATIVE**: `policy.rs`, `config.rs` (security-critical)
- **OPTIONAL**: `constraints.rs`, `hierarchy.rs` (security-relevant)
- **EXPERIMENTAL**: `adaptive.rs` (NOT for production)

### isa-ffi (INFORMATIVE)
- **Security Level**: Binding layer only
- **Note**: Security depends on correct usage from calling language
- **Recommendation**: Validate all inputs at FFI boundary

### isa-merkle (OPTIONAL)
- **Security Level**: Cryptographic proofs
- **Use Case**: Blockchain integration, audit trails
- **Note**: Optional - not required for basic integrity monitoring
