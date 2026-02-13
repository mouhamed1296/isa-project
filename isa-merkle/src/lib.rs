//! Merkle tree batch verification for MA-ISA states.
//!
//! Provides efficient batch verification of multiple device states using
//! Merkle trees with BLAKE3 hashing.
//!
//! ## Use Cases
//!
//! - Verify 1000s of POS terminals efficiently
//! - Aggregate state proofs for multiple devices
//! - Reduce verification overhead in distributed systems
//!
//! ## Example
//!
//! ```rust
//! use isa_merkle::{MerkleTree, StateLeaf};
//! use isa_core::StateVector;
//!
//! // Create test states
//! let state_1 = StateVector {
//!     finance: [1u8; 32],
//!     time: [2u8; 32],
//!     hardware: [3u8; 32],
//! };
//! let state_2 = StateVector {
//!     finance: [4u8; 32],
//!     time: [5u8; 32],
//!     hardware: [6u8; 32],
//! };
//! let state_3 = StateVector {
//!     finance: [7u8; 32],
//!     time: [8u8; 32],
//!     hardware: [9u8; 32],
//! };
//!
//! // Create leaves from device states
//! let leaves: Vec<StateLeaf> = vec![
//!     StateLeaf::new("device_001", state_1),
//!     StateLeaf::new("device_002", state_2),
//!     StateLeaf::new("device_003", state_3),
//! ];
//!
//! // Build Merkle tree
//! let tree = MerkleTree::new(leaves);
//!
//! // Get root hash for verification
//! let root = tree.root();
//!
//! // Generate proof for specific device
//! let proof = tree.prove(0).unwrap();
//!
//! // Verify proof
//! assert!(proof.verify(root));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec, string::String};

use blake3::Hasher;
use isa_core::StateVector;

/// A leaf in the Merkle tree representing a device state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateLeaf {
    /// Device identifier
    pub device_id: String,
    /// State vector (finance, time, hardware)
    pub state: StateVector,
    /// Cached hash of this leaf
    hash: [u8; 32],
}

impl StateLeaf {
    /// Create a new state leaf.
    pub fn new(device_id: impl Into<String>, state: StateVector) -> Self {
        let device_id = device_id.into();
        let hash = Self::compute_hash(&device_id, &state);
        Self {
            device_id,
            state,
            hash,
        }
    }

    /// Compute the hash of a device state.
    fn compute_hash(device_id: &str, state: &StateVector) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(device_id.as_bytes());
        hasher.update(&state.finance);
        hasher.update(&state.time);
        hasher.update(&state.hardware);
        *hasher.finalize().as_bytes()
    }

    /// Get the hash of this leaf.
    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }
}

/// A Merkle tree for batch verification of device states.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MerkleTree {
    /// Leaf nodes (device states)
    leaves: Vec<StateLeaf>,
    /// Internal nodes (hashes)
    nodes: Vec<[u8; 32]>,
    /// Tree height
    height: usize,
}

impl MerkleTree {
    /// Create a new Merkle tree from device states.
    ///
    /// # Panics
    ///
    /// Panics if `leaves` is empty.
    pub fn new(leaves: Vec<StateLeaf>) -> Self {
        assert!(!leaves.is_empty(), "Cannot create empty Merkle tree");

        let leaf_count = leaves.len();
        let height = (leaf_count as f64).log2().ceil() as usize;
        let node_count = (1 << (height + 1)) - 1;

        let mut nodes = vec![[0u8; 32]; node_count];

        // Copy leaf hashes to bottom level
        let leaf_start = (1 << height) - 1;
        for (i, leaf) in leaves.iter().enumerate() {
            nodes[leaf_start + i] = *leaf.hash();
        }

        // Fill remaining leaf positions with duplicates (for non-power-of-2 trees)
        for i in leaf_count..(1 << height) {
            nodes[leaf_start + i] = *leaves[leaf_count - 1].hash();
        }

        // Build tree bottom-up
        for level in (0..height).rev() {
            let level_start = (1 << level) - 1;
            let child_start = (1 << (level + 1)) - 1;

            for i in 0..(1 << level) {
                let left = nodes[child_start + 2 * i];
                let right = nodes[child_start + 2 * i + 1];
                nodes[level_start + i] = Self::hash_pair(&left, &right);
            }
        }

        Self {
            leaves,
            nodes,
            height,
        }
    }

    /// Hash two nodes together.
    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(left);
        hasher.update(right);
        *hasher.finalize().as_bytes()
    }

    /// Get the root hash of the tree.
    pub fn root(&self) -> &[u8; 32] {
        &self.nodes[0]
    }

    /// Get the number of leaves in the tree.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Generate a Merkle proof for a specific leaf index.
    pub fn prove(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut siblings = Vec::new();
        let mut current_index = index;

        for level in (0..self.height).rev() {
            let level_start = (1 << level) - 1;
            let parent_index = current_index / 2;
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            let child_start = (1 << (level + 1)) - 1;
            siblings.push(self.nodes[child_start + sibling_index]);

            current_index = parent_index;
        }

        Some(MerkleProof {
            leaf: self.leaves[index].clone(),
            siblings,
            index,
        })
    }

    /// Verify all leaves in the tree.
    pub fn verify_all(&self) -> bool {
        for i in 0..self.leaves.len() {
            if let Some(proof) = self.prove(i) {
                if !proof.verify(self.root()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

/// A Merkle proof for a single device state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MerkleProof {
    /// The leaf being proven
    pub leaf: StateLeaf,
    /// Sibling hashes along the path to root
    pub siblings: Vec<[u8; 32]>,
    /// Index of the leaf in the tree
    pub index: usize,
}

impl MerkleProof {
    /// Verify this proof against a root hash.
    pub fn verify(&self, root: &[u8; 32]) -> bool {
        let mut current_hash = *self.leaf.hash();
        let mut current_index = self.index;

        for sibling in &self.siblings {
            current_hash = if current_index % 2 == 0 {
                MerkleTree::hash_pair(&current_hash, sibling)
            } else {
                MerkleTree::hash_pair(sibling, &current_hash)
            };
            current_index /= 2;
        }

        &current_hash == root
    }

    /// Get the device ID from this proof.
    pub fn device_id(&self) -> &str {
        &self.leaf.device_id
    }

    /// Get the state vector from this proof.
    pub fn state(&self) -> &StateVector {
        &self.leaf.state
    }
}

/// Batch verification result.
#[derive(Clone, Debug)]
pub struct BatchVerification {
    /// Total number of devices verified
    pub total: usize,
    /// Number of valid devices
    pub valid: usize,
    /// Number of invalid devices
    pub invalid: usize,
    /// Device IDs that failed verification
    pub failed_devices: Vec<String>,
}

impl BatchVerification {
    /// Check if all devices passed verification.
    pub fn all_valid(&self) -> bool {
        self.invalid == 0
    }

    /// Get the success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.valid as f64 / self.total as f64) * 100.0
        }
    }
}

/// Verify a batch of proofs against a root hash.
pub fn verify_batch(proofs: &[MerkleProof], root: &[u8; 32]) -> BatchVerification {
    let total = proofs.len();
    let mut valid = 0;
    let mut failed_devices = Vec::new();

    for proof in proofs {
        if proof.verify(root) {
            valid += 1;
        } else {
            failed_devices.push(proof.device_id().to_string());
        }
    }

    BatchVerification {
        total,
        valid,
        invalid: total - valid,
        failed_devices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state(value: u8) -> StateVector {
        StateVector {
            finance: [value; 32],
            time: [value.wrapping_add(1); 32],
            hardware: [value.wrapping_add(2); 32],
        }
    }

    #[test]
    fn test_single_leaf_tree() {
        let leaves = vec![StateLeaf::new("device_001", create_test_state(1))];
        let tree = MerkleTree::new(leaves);
        assert_eq!(tree.len(), 1);
        assert!(tree.verify_all());
    }

    #[test]
    fn test_multiple_leaves() {
        let leaves = vec![
            StateLeaf::new("device_001", create_test_state(1)),
            StateLeaf::new("device_002", create_test_state(2)),
            StateLeaf::new("device_003", create_test_state(3)),
        ];
        let tree = MerkleTree::new(leaves);
        assert_eq!(tree.len(), 3);
        assert!(tree.verify_all());
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let leaves = vec![
            StateLeaf::new("device_001", create_test_state(1)),
            StateLeaf::new("device_002", create_test_state(2)),
            StateLeaf::new("device_003", create_test_state(3)),
            StateLeaf::new("device_004", create_test_state(4)),
        ];
        let tree = MerkleTree::new(leaves);
        let root = *tree.root();

        for i in 0..tree.len() {
            let proof = tree.prove(i).unwrap();
            assert!(proof.verify(&root));
        }
    }

    #[test]
    fn test_invalid_proof() {
        let leaves = vec![
            StateLeaf::new("device_001", create_test_state(1)),
            StateLeaf::new("device_002", create_test_state(2)),
        ];
        let tree = MerkleTree::new(leaves);
        let root = *tree.root();

        let mut proof = tree.prove(0).unwrap();
        proof.leaf.state.finance[0] = 99; // Tamper with state
        // Recompute hash after tampering
        proof.leaf.hash = StateLeaf::compute_hash(&proof.leaf.device_id, &proof.leaf.state);

        assert!(!proof.verify(&root));
    }

    #[test]
    fn test_batch_verification() {
        let leaves = vec![
            StateLeaf::new("device_001", create_test_state(1)),
            StateLeaf::new("device_002", create_test_state(2)),
            StateLeaf::new("device_003", create_test_state(3)),
        ];
        let tree = MerkleTree::new(leaves);
        let root = *tree.root();

        let proofs: Vec<_> = (0..tree.len())
            .map(|i| tree.prove(i).unwrap())
            .collect();

        let result = verify_batch(&proofs, &root);
        assert!(result.all_valid());
        assert_eq!(result.success_rate(), 100.0);
    }

    #[test]
    fn test_batch_with_failures() {
        let leaves = vec![
            StateLeaf::new("device_001", create_test_state(1)),
            StateLeaf::new("device_002", create_test_state(2)),
            StateLeaf::new("device_003", create_test_state(3)),
        ];
        let tree = MerkleTree::new(leaves);
        let root = *tree.root();

        let mut proofs: Vec<_> = (0..tree.len())
            .map(|i| tree.prove(i).unwrap())
            .collect();

        // Tamper with one proof
        proofs[1].leaf.state.finance[0] = 99;
        // Recompute hash after tampering
        proofs[1].leaf.hash = StateLeaf::compute_hash(&proofs[1].leaf.device_id, &proofs[1].leaf.state);

        let result = verify_batch(&proofs, &root);
        assert!(!result.all_valid());
        assert_eq!(result.valid, 2);
        assert_eq!(result.invalid, 1);
        assert_eq!(result.failed_devices, vec!["device_002"]);
    }

    #[test]
    fn test_power_of_two_leaves() {
        for size in [1, 2, 4, 8, 16, 32] {
            let leaves: Vec<_> = (0..size)
                .map(|i| StateLeaf::new(format!("device_{:03}", i), create_test_state(i as u8)))
                .collect();
            let tree = MerkleTree::new(leaves);
            assert!(tree.verify_all());
        }
    }

    #[test]
    fn test_non_power_of_two_leaves() {
        for size in [3, 5, 7, 10, 15, 20] {
            let leaves: Vec<_> = (0..size)
                .map(|i| StateLeaf::new(format!("device_{:03}", i), create_test_state(i as u8)))
                .collect();
            let tree = MerkleTree::new(leaves);
            assert!(tree.verify_all());
        }
    }
}
