//! Performance benchmarks for MA-ISA core operations.
//!
//! Run with: cargo bench
//! View HTML reports in: target/criterion/report/index.html

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use isa_core::{AxisAccumulator, MultiAxisState, CircularDistance};

/// Benchmark single-axis accumulation
fn bench_axis_accumulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("axis_accumulation");
    
    let seed = [0u8; 32];
    let event = b"sale:1000.00";
    let entropy = b"device:pos_terminal_001";
    
    group.bench_function("single_accumulate", |b| {
        b.iter(|| {
            let mut axis = AxisAccumulator::new(black_box(seed));
            axis.accumulate(black_box(event), black_box(entropy), black_box(100));
            black_box(axis.state())
        });
    });
    
    group.bench_function("100_sequential_accumulates", |b| {
        b.iter(|| {
            let mut axis = AxisAccumulator::new(black_box(seed));
            for i in 0..100 {
                axis.accumulate(black_box(event), black_box(entropy), black_box(i));
            }
            black_box(axis.state())
        });
    });
    
    group.finish();
}

/// Benchmark multi-axis state operations
fn bench_multi_axis_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_axis_state");
    
    let master_seed = [42u8; 32];
    
    group.bench_function("from_master_seed", |b| {
        b.iter(|| {
            black_box(MultiAxisState::from_master_seed(black_box(master_seed)))
        });
    });
    
    group.bench_function("state_vector", |b| {
        let state = MultiAxisState::from_master_seed(master_seed);
        b.iter(|| {
            black_box(state.state_vector())
        });
    });
    
    group.bench_function("divergence_calculation", |b| {
        let state1 = MultiAxisState::from_master_seed([1u8; 32]);
        let state2 = MultiAxisState::from_master_seed([2u8; 32]);
        b.iter(|| {
            black_box(state1.divergence(black_box(&state2)))
        });
    });
    
    group.finish();
}

/// Benchmark circular distance calculations
fn bench_circular_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("circular_distance");
    
    let state_a = [0x42u8; 32];
    let state_b = [0x84u8; 32];
    
    group.bench_function("compute", |b| {
        b.iter(|| {
            black_box(CircularDistance::compute(black_box(&state_a), black_box(&state_b)))
        });
    });
    
    group.bench_function("compute_scalar", |b| {
        b.iter(|| {
            black_box(CircularDistance::compute_scalar(black_box(&state_a), black_box(&state_b)))
        });
    });
    
    group.bench_function("compare", |b| {
        b.iter(|| {
            black_box(CircularDistance::compare(black_box(&state_a), black_box(&state_b)))
        });
    });
    
    group.bench_function("compare_scalar", |b| {
        b.iter(|| {
            black_box(CircularDistance::compare_scalar(black_box(&state_a), black_box(&state_b)))
        });
    });
    
    group.bench_function("min_distance", |b| {
        b.iter(|| {
            black_box(CircularDistance::min_distance(black_box(&state_a), black_box(&state_b)))
        });
    });
    
    group.finish();
}

/// Benchmark serialization/deserialization
#[cfg(feature = "serde")]
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let master_seed = [42u8; 32];
    let state = MultiAxisState::from_master_seed(master_seed);
    
    group.bench_function("to_bytes", |b| {
        b.iter(|| {
            black_box(state.to_bytes().unwrap())
        });
    });
    
    let bytes = state.to_bytes().unwrap();
    group.bench_function("from_bytes", |b| {
        b.iter(|| {
            black_box(MultiAxisState::from_bytes(black_box(&bytes)).unwrap())
        });
    });
    
    group.bench_function("roundtrip", |b| {
        b.iter(|| {
            let bytes = state.to_bytes().unwrap();
            black_box(MultiAxisState::from_bytes(&bytes).unwrap())
        });
    });
    
    group.finish();
}

/// Benchmark with varying event sizes
fn bench_event_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_sizes");
    
    let seed = [0u8; 32];
    let entropy = b"fixed_entropy";
    
    for size in [16, 64, 256, 1024, 4096].iter() {
        let event = vec![0x42u8; *size];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut axis = AxisAccumulator::new(seed);
                axis.accumulate(black_box(&event), black_box(entropy), black_box(1));
                black_box(axis.state())
            });
        });
    }
    
    group.finish();
}

/// Benchmark with varying entropy sizes
fn bench_entropy_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("entropy_sizes");
    
    let seed = [0u8; 32];
    let event = b"fixed_event";
    
    for size in [16, 32, 64, 128, 256].iter() {
        let entropy = vec![0x42u8; *size];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut axis = AxisAccumulator::new(seed);
                axis.accumulate(black_box(event), black_box(&entropy), black_box(1));
                black_box(axis.state())
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_axis_accumulation,
    bench_multi_axis_state,
    bench_circular_distance,
    bench_event_sizes,
    bench_entropy_sizes,
);

#[cfg(feature = "serde")]
criterion_group!(
    serde_benches,
    bench_serialization,
);

#[cfg(feature = "serde")]
criterion_main!(benches, serde_benches);

#[cfg(not(feature = "serde"))]
criterion_main!(benches);
