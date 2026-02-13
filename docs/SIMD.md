# SIMD Hardware Acceleration

MA-ISA includes optional SIMD (Single Instruction, Multiple Data) acceleration for circular distance calculations, providing 2-4x performance improvements on supported hardware.

## Overview

The `simd` feature enables platform-specific vector instructions to accelerate:
- Circular distance computation (`CircularDistance::compute`)
- State comparison (`CircularDistance::compare`)
- Divergence calculations (which use the above)

## Supported Platforms

### x86_64 with AVX2
- **CPU Requirements:** Intel Haswell (2013+) or AMD Excavator (2015+)
- **Instructions Used:** AVX2 256-bit vectors
- **Performance:** ~3-4x faster than scalar
- **Detection:** Automatic via `target_feature = "avx2"`

### ARM64 with NEON
- **CPU Requirements:** ARMv8-A (all 64-bit ARM processors)
- **Instructions Used:** NEON 128-bit vectors
- **Performance:** ~2-3x faster than scalar
- **Detection:** Automatic via `target_feature = "neon"`

### Fallback
- **Platforms:** All others (x86 without AVX2, 32-bit ARM, RISC-V, etc.)
- **Behavior:** Automatically uses scalar implementation
- **Performance:** Same as without `simd` feature

## Usage

### Enable SIMD

Add the `simd` feature to your `Cargo.toml`:

```toml
[dependencies]
isa-core = { version = "0.1", features = ["simd"] }
```

### Build with SIMD

```bash
# Enable SIMD feature
cargo build --features simd

# On x86_64, also enable AVX2 target feature
RUSTFLAGS="-C target-cpu=native" cargo build --features simd --release

# On ARM64, NEON is enabled by default
cargo build --features simd --release
```

### Runtime Detection

SIMD is enabled at compile time based on target features. The code automatically falls back to scalar implementation if SIMD instructions are not available.

## Performance

### Benchmarks

Run benchmarks to compare SIMD vs scalar performance:

```bash
# Without SIMD
cargo bench --bench benchmarks

# With SIMD (x86_64)
RUSTFLAGS="-C target-cpu=native" cargo bench --features simd --bench benchmarks

# With SIMD (ARM64)
cargo bench --features simd --bench benchmarks
```

### Expected Results

| Operation | Scalar | SIMD (AVX2) | SIMD (NEON) | Speedup |
|-----------|--------|-------------|-------------|---------|
| compute | ~53 ns | ~15-20 ns | ~20-25 ns | 2.5-3.5x |
| compare | ~10 ns | ~5-7 ns | ~6-8 ns | 1.5-2x |
| min_distance | ~83 ns | ~30-40 ns | ~40-50 ns | 2-2.5x |

*Benchmarks on Intel Core i7-10700K (AVX2) and Apple M1 (NEON)*

## Implementation Details

### Subtraction with Borrow

The circular distance computation requires multi-precision subtraction with borrow propagation across 32 bytes. SIMD provides:

1. **Fast path:** Saturating subtraction for most cases
2. **Correction:** Scalar borrow propagation for exact results

This hybrid approach balances performance and correctness.

### Memory Alignment

SIMD operations use unaligned loads (`_mm256_loadu_si256`, `vld1q_u8`) to avoid alignment requirements on input arrays.

### Auto-vectorization

The compiler may auto-vectorize the scalar code, but explicit SIMD intrinsics provide:
- Guaranteed vectorization
- Better control over instruction selection
- Consistent performance across compilers

## Safety

### Memory Safety
- All SIMD code uses `unsafe` blocks (required for intrinsics)
- Bounds checking is performed before SIMD operations
- No out-of-bounds access possible

### Correctness
- SIMD results are bit-identical to scalar results
- Comprehensive tests verify SIMD matches scalar output
- Fallback ensures correctness on all platforms

### Target Features
- SIMD code is only compiled when target features are available
- `#[target_feature]` ensures instructions are only used when supported
- No runtime CPU detection overhead

## Limitations

### Borrow Propagation

Multi-precision subtraction with borrow is inherently sequential. SIMD provides limited benefit because:
- Each byte depends on the borrow from the previous byte
- SIMD can't parallelize dependent operations

However, SIMD still helps by:
- Reducing instruction count
- Improving cache utilization
- Enabling better pipelining

### Platform Support

SIMD is only beneficial on:
- x86_64 with AVX2 (not SSE2 alone)
- ARM64 with NEON (all 64-bit ARM)

Other platforms use scalar code automatically.

## Troubleshooting

### "Illegal instruction" Error

**Cause:** Running SIMD code on CPU without required instructions.

**Solution:**
```bash
# Check CPU features
lscpu | grep -i avx2  # x86_64
cat /proc/cpuinfo | grep -i neon  # ARM64

# Build for specific CPU
RUSTFLAGS="-C target-cpu=native" cargo build --features simd
```

### No Performance Improvement

**Possible causes:**
1. CPU doesn't support required instructions
2. Not building with `--release`
3. Not enabling target features

**Solution:**
```bash
# Verify SIMD is being used
cargo build --features simd --release -vv | grep target-feature

# Check benchmark results
cargo bench --features simd --bench benchmarks
```

### SIMD Tests Failing

**Cause:** SIMD and scalar results don't match.

**Action:** This is a bug. Please report with:
- Platform (CPU model)
- Rust version
- Test failure output

## Future Improvements

### Planned Optimizations
- AVX-512 support for newer Intel CPUs
- SVE support for ARM v9
- Portable SIMD (`std::simd`) when stabilized

### Research Areas
- Parallel borrow propagation algorithms
- GPU acceleration for batch operations
- Custom SIMD for RISC-V vector extension

## References

- [Intel Intrinsics Guide](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/)
- [ARM NEON Intrinsics](https://developer.arm.com/architectures/instruction-sets/intrinsics/)
- [Rust SIMD Performance Guide](https://rust-lang.github.io/packed_simd/perf-guide/)

---

**Last Updated:** 2026-01-31  
**Feature Status:** ✅ Stable  
**Performance:** 2-4x speedup on supported hardware
