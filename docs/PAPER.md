# Integral State Accumulators: Continuous Integrity for Offline Adversarial Systems

**Author:** Mamadou Sarr  
**Date:** February 1, 2026

See full paper content in academic format.

## Implementation Reference

This repository contains the production implementation of the ISA primitive described in the paper.

### Code Mapping

- **Equation (1) - State Accumulation:** `isa-core/src/axis.rs::AxisAccumulator::accumulate()`
- **Equation (2) - KDF Function:** `isa-core/src/kdf.rs::derive_axis_seed()`
- **Equation (7) - Circular Distance:** `isa-core/src/divergence.rs::CircularDistance::compute()`
- **Theorem 1 - Forgery Resistance:** Validated in `isa-core/src/verify.rs` (Kani proofs)
- **Multi-Axis State:** `isa-core/src/lib.rs::MultiAxisState`

### Experimental Validation

All results in Table 1 are reproducible via:
```bash
cargo test --all
cargo bench
```

### Citation

```bibtex
@article{sarr2026isa,
  title={Integral State Accumulators: Continuous Integrity for Offline Adversarial Systems},
  author={Sarr, Mamadou},
  year={2026},
  note={Pre-print}
}
```
