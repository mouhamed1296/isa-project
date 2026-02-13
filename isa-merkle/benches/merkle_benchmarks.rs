//! Benchmarks for Merkle tree batch verification.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use isa_merkle::{MerkleTree, StateLeaf, verify_batch};
use isa_core::StateVector;

fn create_test_state(value: u8) -> StateVector {
    StateVector {
        finance: [value; 32],
        time: [value.wrapping_add(1); 32],
        hardware: [value.wrapping_add(2); 32],
    }
}

fn bench_tree_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_construction");

    for size in [10, 100, 1000, 10000].iter() {
        let leaves: Vec<_> = (0..*size)
            .map(|i| StateLeaf::new(format!("device_{:05}", i), create_test_state((i % 256) as u8)))
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let tree = MerkleTree::new(black_box(leaves.clone()));
                black_box(tree)
            });
        });
    }

    group.finish();
}

fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generation");

    for size in [10, 100, 1000, 10000].iter() {
        let leaves: Vec<_> = (0..*size)
            .map(|i| StateLeaf::new(format!("device_{:05}", i), create_test_state((i % 256) as u8)))
            .collect();
        let tree = MerkleTree::new(leaves);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let proof = tree.prove(black_box(*size / 2));
                black_box(proof)
            });
        });
    }

    group.finish();
}

fn bench_proof_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_verification");

    for size in [10, 100, 1000, 10000].iter() {
        let leaves: Vec<_> = (0..*size)
            .map(|i| StateLeaf::new(format!("device_{:05}", i), create_test_state((i % 256) as u8)))
            .collect();
        let tree = MerkleTree::new(leaves);
        let root = *tree.root();
        let proof = tree.prove(*size / 2).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let valid = proof.verify(black_box(&root));
                black_box(valid)
            });
        });
    }

    group.finish();
}

fn bench_batch_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_verification");

    for size in [10, 100, 1000].iter() {
        let leaves: Vec<_> = (0..*size)
            .map(|i| StateLeaf::new(format!("device_{:05}", i), create_test_state((i % 256) as u8)))
            .collect();
        let tree = MerkleTree::new(leaves);
        let root = *tree.root();
        let proofs: Vec<_> = (0..*size)
            .map(|i| tree.prove(i).unwrap())
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = verify_batch(black_box(&proofs), black_box(&root));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_full_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_workflow");

    for size in [10, 100, 1000].iter() {
        let leaves: Vec<_> = (0..*size)
            .map(|i| StateLeaf::new(format!("device_{:05}", i), create_test_state((i % 256) as u8)))
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                // Build tree
                let tree = MerkleTree::new(black_box(leaves.clone()));
                let root = *tree.root();

                // Generate proofs
                let proofs: Vec<_> = (0..*size)
                    .map(|i| tree.prove(i).unwrap())
                    .collect();

                // Verify batch
                let result = verify_batch(&proofs, &root);
                black_box(result)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tree_construction,
    bench_proof_generation,
    bench_proof_verification,
    bench_batch_verification,
    bench_full_workflow,
);

criterion_main!(benches);
