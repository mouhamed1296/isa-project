# MA-ISA Conformance Statement

## Document Status

**Version:** 0.1.0  
**Status:** Draft for ISO/IEC Standardization Review  
**Date:** 2026-02-03

## 1. Scope

This document defines conformance requirements for implementations of the Multi-Axis Integral State Accumulator (MA-ISA) integrity monitoring system.

### 1.1 Normative Scope

MA-ISA standardizes:
- Deterministic integrity dimension accumulation
- Multi-dimensional state management
- Divergence calculation between states
- Runtime-configurable dimension support
- Threshold-based integrity evaluation

### 1.2 Out of Scope

The following are explicitly **NOT** part of the normative specification:
- Specific recovery/reconciliation mechanisms
- Machine learning or adaptive optimization
- Hierarchical dimension organization
- Cross-dimension constraint enforcement
- Platform-specific entropy sources
- Persistence formats
- Language bindings

## 2. Conformance Classes

### 2.1 MA-ISA Core Conformance

An implementation conforms to **MA-ISA Core** if and only if it:

1. **SHALL** implement all types and functions defined in `isa-core`:
   - `AxisAccumulator` - Single dimension accumulation primitive
   - `DimensionAccumulator` - Integrity dimension wrapper
   - `IntegrityState<N>` - Fixed-dimension state management
   - `DynamicIntegrityState` - Runtime-configurable dimensions
   - Divergence calculation via `CircularDistance`
   - Key derivation via `Kdf`

2. **SHALL** maintain deterministic behavior:
   - Same inputs produce same outputs
   - No use of randomness, time, or I/O in core operations
   - Platform-independent results

3. **SHALL** implement threshold evaluation:
   - Per-dimension divergence thresholds
   - Threshold violation detection

4. **SHALL** support configuration loading:
   - Read dimension policies from external sources
   - Parse threshold values and dimension metadata

### 2.2 Extended Conformance (Optional)

Implementations **MAY** additionally support:

- **Cross-dimension constraints** (`isa-runtime/constraints`)
  - Ratio constraints between dimensions
  - Sum-based aggregate constraints
  - Conditional constraint evaluation

- **Dimension hierarchies** (`isa-runtime/hierarchy`)
  - Parent-child dimension relationships
  - Weighted aggregation
  - Tree-based organization

These extensions **SHALL NOT** be required for basic conformance.

### 2.3 Experimental Features (Non-Normative)

The following features are **EXPERIMENTAL** and **NOT RECOMMENDED** for production:

- **Adaptive profiles** (`isa-runtime/adaptive`)
  - Machine learning-based weight optimization
  - Automatic parameter estimation
  - Historical observation tracking

Implementations **SHALL NOT** rely on experimental features for correctness or safety properties.

## 3. Normative Requirements

### 3.1 State Accumulation

Implementations **SHALL**:
- Use cryptographic mixing for state updates
- Maintain irreversible state evolution
- Support wrapping counter increments without panics
- Preserve state integrity across serialization/deserialization

### 3.2 Divergence Calculation

Implementations **SHALL**:
- Calculate divergence as circular distance in state space
- Support per-dimension divergence measurement
- Provide aggregate divergence metrics
- Use consistent distance metrics across all dimensions

### 3.3 Threshold Evaluation

Implementations **SHALL**:
- Support per-dimension threshold configuration
- Detect threshold violations deterministically
- Report which dimensions exceeded thresholds
- Allow runtime threshold updates

### 3.4 Configuration

Implementations **SHALL**:
- Support external configuration loading (YAML, JSON, TOML, or environment variables)
- Parse dimension metadata (name, threshold, weight)
- Validate configuration before use
- Provide clear error messages for invalid configuration

## 4. Implementation Recommendations

### 4.1 Reconciliation Strategies (Informative)

While not normative, implementations **SHOULD** consider:
- Immediate reconciliation for safety-relevant dimensions
- Monitor-only mode for diagnostic dimensions
- Quarantine mechanisms for compromised dimensions
- Full system recovery for catastrophic divergence

The specific reconciliation mechanism is implementation-defined.

### 4.2 Performance Considerations (Informative)

Implementations **SHOULD**:
- Optimize for low-latency divergence calculation
- Minimize memory allocation in hot paths
- Support no_std environments where applicable
- Provide batch processing for multiple events

### 4.3 Security Considerations (Informative)

Implementations **SHOULD**:
- Use hardware entropy sources when available
- Protect master seeds with appropriate key management
- Zeroize sensitive state on drop
- Avoid timing side channels in cryptographic operations

## 5. Conformance Testing

### 5.1 Test Vectors

Conforming implementations **SHALL** pass all test vectors defined in:
- `isa-core/tests/` - Core accumulation and divergence tests
- `isa-runtime/tests/` - Policy and configuration tests

### 5.2 Determinism Tests

Implementations **SHALL** demonstrate:
- Identical results across multiple runs with same inputs
- Platform-independent behavior
- Reproducible serialization/deserialization

### 5.3 Interoperability Tests

Implementations **SHALL** demonstrate:
- Compatible state serialization formats
- Consistent divergence calculations
- Equivalent threshold evaluation results

## 6. Conformance Claim

An implementation **MAY** claim conformance by stating:

> "This implementation conforms to MA-ISA Core v0.1.0 as defined in the MA-ISA specification. It implements all normative requirements defined in Sections 3.1-3.4 and passes all conformance tests defined in Section 5."

Optional extensions **MAY** be listed separately:

> "This implementation additionally supports the following optional extensions: [list extensions]"

Experimental features **SHALL NOT** be mentioned in conformance claims.

## 7. Terminology

### 7.1 Normative Terms

- **Integrity Dimension**: A single axis of integrity monitoring (replaces informal "axis")
- **State Reconciliation**: Process of restoring integrity after divergence (replaces "healing")
- **Safety-Relevant Dimension**: A dimension critical for system safety (replaces "critical dimension")
- **Operational Confidence Level**: Degree of trust in system integrity (replaces "trust zone")
- **Divergence Threshold**: Maximum acceptable divergence before triggering reconciliation

### 7.2 RFC 2119 Keywords

This specification uses RFC 2119 keywords:
- **SHALL** / **SHALL NOT**: Mandatory requirement
- **SHOULD** / **SHOULD NOT**: Recommended but not mandatory
- **MAY**: Optional, at implementer's discretion

## 8. References

### 8.1 Normative References

- RFC 2119: Key words for use in RFCs to Indicate Requirement Levels
- FIPS 202: SHA-3 Standard (for cryptographic mixing)

### 8.2 Informative References

- MA-ISA Implementation Guide
- MA-ISA Enhancements Documentation
- Configuration Examples

## 9. Revision History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-02-03 | Initial draft for ISO review |

---

**Document Classification**: Normative  
**Intended Audience**: Implementers, Standards Bodies, Certification Authorities
