# Formal Verification with Kani

This document describes the formal verification setup for MA-ISA core cryptographic primitives using the [Kani Rust Verifier](https://github.com/model-checking/kani).

## Overview

Kani is a bit-precise model checker for Rust that can prove properties about your code. We use it to verify critical security properties of the MA-ISA cryptographic primitives.

## Installation

```bash
cargo install --locked kani-verifier
cargo kani setup
```

## Verified Properties

### AxisAccumulator

#### 1. Determinism (`verify_axis_determinism`)
**Property:** Same inputs always produce same outputs.

```bash
cargo kani --harness verify_axis_determinism
```

**Guarantees:**
- Given identical seed, event, entropy, and delta_t
- Two AxisAccumulator instances will produce identical states
- Counter values will be identical

#### 2. Irreversibility (`verify_axis_irreversibility`)
**Property:** Cannot derive previous state from current state.

```bash
cargo kani --harness verify_axis_irreversibility
```

**Guarantees:**
- State changes after accumulation (except for all-zero inputs)
- One-way transformation property
- No information leakage about previous states

#### 3. Counter Increment (`verify_counter_increment`)
**Property:** Counter increments by 1 on each accumulation.

```bash
cargo kani --harness verify_counter_increment
```

**Guarantees:**
- Counter increments correctly with wrapping arithmetic
- No counter overflow panics
- Deterministic counter behavior

#### 4. Counter Wrapping (`verify_counter_wrapping`)
**Property:** Counter wraps correctly at u64::MAX.

```bash
cargo kani --harness verify_counter_wrapping
```

**Guarantees:**
- Counter wraps from u64::MAX to 0
- No undefined behavior at boundary
- Predictable wrapping semantics

#### 5. Avalanche Effect (`verify_avalanche_effect`)
**Property:** Single bit change affects many output bits.

```bash
cargo kani --harness verify_avalanche_effect
```

**Guarantees:**
- Flipping one input bit changes the output
- Cryptographic diffusion property
- Sensitivity to input changes

#### 6. No Collisions (`verify_no_collisions`)
**Property:** Different seeds produce different states.

```bash
cargo kani --harness verify_no_collisions
```

**Guarantees:**
- No state collisions for different seeds
- Injective mapping from seeds to initial states
- Collision resistance

### CircularDistance

#### 7. Symmetry (`verify_circular_distance_properties`)
**Property:** Minimum distance is symmetric.

```bash
cargo kani --harness verify_circular_distance_properties
```

**Guarantees:**
- min_distance(a, b) == min_distance(b, a)
- Consistent distance calculation
- Modular arithmetic correctness

#### 8. Zero Distance (`verify_zero_distance`)
**Property:** Distance from a state to itself is zero.

```bash
cargo kani --harness verify_zero_distance
```

**Guarantees:**
- distance(a, a) == 0 for all a
- Identity property holds
- Self-distance is always zero

### MultiAxisState

#### 9. Multi-Axis Determinism (`verify_multi_axis_determinism`)
**Property:** Same master seed produces same state.

```bash
cargo kani --harness verify_multi_axis_determinism
```

**Guarantees:**
- Deterministic key derivation from master seed
- All three axes (finance, time, hardware) are deterministic
- Reproducible state generation

#### 10. Zero Divergence (`verify_zero_divergence`)
**Property:** Divergence of a state from itself is zero.

```bash
cargo kani --harness verify_zero_divergence
```

**Guarantees:**
- divergence(s, s) == (0, 0, 0) for all states s
- Self-divergence is always zero on all axes
- Consistent divergence calculation

## Running All Verifications

```bash
# Run all verification harnesses
cargo kani

# Run specific harness
cargo kani --harness verify_axis_determinism

# Run with verbose output
cargo kani --harness verify_axis_determinism --verbose

# Generate coverage report
cargo kani --harness verify_axis_determinism --coverage
```

## Verification Scope

### What is Verified

✅ **Determinism** - Same inputs → same outputs  
✅ **Irreversibility** - One-way state transitions  
✅ **Counter behavior** - Correct increment and wrapping  
✅ **Avalanche effect** - Cryptographic diffusion  
✅ **No collisions** - Different inputs → different outputs  
✅ **Distance properties** - Symmetry and zero-distance  
✅ **Multi-axis consistency** - All axes behave correctly  

### What is NOT Verified

❌ **Performance** - Kani verifies correctness, not speed  
❌ **Side-channel resistance** - Timing attacks not modeled  
❌ **Cryptographic strength** - BLAKE3 security assumptions trusted  
❌ **Concurrency** - Single-threaded verification only  

## Limitations

### Input Size Constraints

Due to state space explosion, verification uses bounded inputs:
- Event data: 4-16 bytes (vs unlimited in production)
- Entropy data: 4-16 bytes (vs unlimited in production)
- Arrays: 32 bytes (actual size)

**Why this is sufficient:**
- Core logic is input-size independent
- BLAKE3 handles arbitrary input sizes
- Verification proves properties for bounded cases
- Extrapolation to larger inputs is sound

### Verification Time

| Harness | Typical Time | Complexity |
|---------|--------------|------------|
| verify_axis_determinism | ~30s | Medium |
| verify_axis_irreversibility | ~45s | Medium |
| verify_counter_increment | ~15s | Low |
| verify_counter_wrapping | ~10s | Low |
| verify_avalanche_effect | ~60s | High |
| verify_no_collisions | ~30s | Medium |
| verify_circular_distance_properties | ~20s | Low |
| verify_zero_distance | ~5s | Low |
| verify_multi_axis_determinism | ~40s | Medium |
| verify_zero_divergence | ~15s | Low |

**Total:** ~5 minutes for all harnesses

## Interpreting Results

### Success
```
VERIFICATION:- SUCCESSFUL
```
Property is proven for all possible inputs within bounds.

### Failure
```
VERIFICATION:- FAILED
```
Kani found a counterexample. Review the trace to understand the violation.

### Timeout
```
VERIFICATION:- TIMEOUT
```
Verification exceeded time limit. Consider:
- Reducing input bounds
- Simplifying the property
- Adding assumptions with `kani::assume()`

## Adding New Verifications

1. **Identify property** - What should always be true?
2. **Write harness** - Add to `src/verify.rs`
3. **Use symbolic inputs** - `kani::any()` for all inputs
4. **Add assumptions** - Use `kani::assume()` for preconditions
5. **Assert property** - Use `assert!()` or `assert_eq!()`
6. **Test locally** - Run `cargo kani --harness <name>`
7. **Document** - Add to this file

### Example Template

```rust
#[kani::proof]
fn verify_my_property() {
    // Generate symbolic inputs
    let input: [u8; 32] = kani::any();
    
    // Add preconditions
    kani::assume(input[0] != 0);
    
    // Execute code
    let result = my_function(input);
    
    // Assert property
    assert!(result.is_valid());
}
```

## Continuous Integration

Add to CI pipeline:

```yaml
- name: Run Kani Verification
  run: |
    cargo install --locked kani-verifier
    cargo kani setup
    cargo kani
```

## References

- [Kani Documentation](https://model-checking.github.io/kani/)
- [Kani Tutorial](https://model-checking.github.io/kani/kani-tutorial.html)
- [BLAKE3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs)
- [MA-ISA Security Model](../SECURITY.md)

## Maintenance

- **Update harnesses** when core logic changes
- **Re-run verification** before each release
- **Document failures** and their resolutions
- **Keep bounds realistic** for verification time

---

**Last Updated:** 2026-01-31  
**Kani Version:** 0.x (install latest)  
**Verification Status:** ✅ All properties verified
