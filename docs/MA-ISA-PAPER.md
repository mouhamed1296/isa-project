# Multi-Axis Integral State Accumulators: Vectorized Integrity and Policy-Composable Trust

**Author:** Mamadou Sarr  
**Affiliation:** Génie Logiciel, Dakar, Senegal  
**Date:** February 1, 2026

## Abstract

Modern offline systems require multi-dimensional integrity guarantees where financial, temporal, and hardware states evolve independently. We propose **Multi-Axis Integral State Accumulators (MA-ISA)**, a vectorized generalization of the ISA framework. By representing system state as a vector of modular integrals **S⃗ ∈ Z^m_2^256**, we enable granular risk assessment and partial recovery. We provide formal proofs for axis isolation and construct an efficient Zero-Knowledge (ZK) protocol for vectorized divergence attestation.

## 1. Introduction

Current integrity primitives, such as Merkle Trees or simple Hash Chains, treat integrity as a binary scalar: a system is either "valid" or "broken." This model fails in complex environments where a device may have perfect financial integrity but compromised temporal synchronization.

This paper introduces a multi-dimensional approach, treating trust as a vector in a modular manifold.

## 2. System Model

### 2.1 The MA-ISA State Vector

Let m be the number of independent integrity axes. The global state **S⃗ₙ** at step n is defined as:

```
S⃗ₙ = (S⁽¹⁾ₙ, S⁽²⁾ₙ, ..., S⁽ᵐ⁾ₙ) ∈ Z^m_M     (1)
```

where M = 2²⁵⁶. Each axis j evolves according to its own contribution function Φ⁽ʲ⁾ and entropy source η⁽ʲ⁾:

```
S⁽ʲ⁾ₙ = (S⁽ʲ⁾ₙ₋₁ + Φ⁽ʲ⁾(eₙ, tₙ, η⁽ʲ⁾ₙ)) (mod M)     (2)
```

## 3. Formal Proofs

### 3.1 Axis Isolation Theorem

**Theorem 3.1 (Axis Isolation):** An adversary A who compromises axis j cannot induce a divergence Dₖ > ε on any axis k ≠ j (where ε is a negligible security parameter), provided the entropy sources η⁽ʲ⁾ and η⁽ᵏ⁾ are statistically independent.

**Proof:** Let axis k be defined by the accumulation of values salted by η⁽ᵏ⁾. The state S⁽ᵏ⁾ is a result of a recursive KDF chain: S⁽ᵏ⁾ₙ = KDF(event ∥ η⁽ᵏ⁾ₙ). If η⁽ʲ⁾ is independent of η⁽ᵏ⁾, then for any modification of events on axis j, the inputs to the KDF on axis k remain unchanged.

Since the KDF is a cryptographically secure pseudo-random function, the output distribution of axis k remains statistically indistinguishable from a random walk in Z_M. Thus, the divergence Dₖ remains bounded by the honest drift τ_honest, and the probability of a forgery on k is 1/M, which is negligible. □

## 4. MA-ISA-ZK Protocol

We define a Zero-Knowledge relation R where the Prover convinces the Verifier that the divergence vector **D⃗ = (dist(S⁽ʲ⁾_L, S⁽ʲ⁾_R))ᵐⱼ₌₁** satisfies **D⃗ ≤ τ⃗** element-wise.

### 4.1 Completeness

If the Prover is honest and ∀j, Dⱼ ≤ τⱼ, the algebraic constraints in the arithmetic circuit (modular subtraction and range checks) will evaluate to 1, and the verifier will accept the proof with probability 1.

### 4.2 Soundness

If ∃j such that Dⱼ > τⱼ, the Prover must produce a witness S⁽ʲ⁾_L that satisfies the circular distance constraint. By the collision-resistance of the underlying commitment scheme, finding such a witness is as difficult as finding a hash collision, ensuring that invalid states are rejected.

## 5. Conclusion

MA-ISA provides a resilient, multi-dimensional framework for trust. By decoupling integrity axes and applying Zero-Knowledge proofs, we create a system that is both auditable and private.

---

## Implementation Notes

This paper describes the theoretical framework for the **Multi-Axis** extension of ISA. The production implementation can be found in the `isa-core` crate:

- **State Vector:** `MultiAxisState` with three axes (finance, time, hardware)
- **Axis Isolation:** Independent `AxisAccumulator` instances with separate entropy
- **Vectorized Divergence:** `DivergenceMetric` computes per-axis distances
- **Recovery Protocol:** Per-axis convergence constants in `isa-runtime`

### Citation

```bibtex
@article{sarr2026maisa,
  title={Multi-Axis Integral State Accumulators: Vectorized Integrity and Policy-Composable Trust},
  author={Sarr, Mamadou},
  institution={Génie Logiciel},
  address={Dakar, Senegal},
  year={2026},
  note={Pre-print}
}
```

## References

1. Sarr, M. (2026). "Integral State Accumulators: Continuous Integrity for Offline Adversarial Systems." [PAPER.md](PAPER.md)
2. Implementation: https://github.com/[username]/isa-project
3. Groth16 ZK-SNARK: "On the Size of Pairing-based Non-interactive Arguments." Eurocrypt 2016.
4. Plonk: "Permutations over Lagrange-bases for Oecumenical Noninteractive arguments of Knowledge." ePrint 2019/953.
