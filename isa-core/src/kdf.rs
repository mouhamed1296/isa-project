//! Deterministic key derivation functions.
//!
//! BLAKE3-based KDF for deriving keys and mixing state.
//!
//! ## Invariants
//!
//! - All derivations are deterministic
//! - Context strings provide domain separation
//! - No randomness or external entropy

use blake3::Hasher;

pub struct Kdf {
    hasher: Hasher,
}

impl Kdf {
    pub fn new(context: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"MA-ISA-KDF-v1");
        hasher.update(context);
        Self { hasher }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub fn finalize(self) -> [u8; 32] {
        let hash = self.hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(hash.as_bytes());
        output
    }

    pub fn derive_key(context: &[u8], inputs: &[&[u8]]) -> [u8; 32] {
        let mut kdf = Self::new(context);
        for input in inputs {
            kdf.update(input);
        }
        kdf.finalize()
    }
}

impl Drop for Kdf {
    fn drop(&mut self) {
    }
}

pub fn mix_state(state: &[u8; 32], event: &[u8], entropy: &[u8], delta_t: u64) -> [u8; 32] {
    let delta_bytes = delta_t.to_le_bytes();
    Kdf::derive_key(b"axis-accumulate", &[state, event, entropy, &delta_bytes])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_deterministic() {
        let key1 = Kdf::derive_key(b"test", &[b"input1", b"input2"]);
        let key2 = Kdf::derive_key(b"test", &[b"input1", b"input2"]);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_kdf_different_context() {
        let key1 = Kdf::derive_key(b"context1", &[b"input"]);
        let key2 = Kdf::derive_key(b"context2", &[b"input"]);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_mix_state_deterministic() {
        let state = [0u8; 32];
        let event = b"sale_event";
        let entropy = b"entropy_source";
        let delta_t = 1000u64;

        let result1 = mix_state(&state, event, entropy, delta_t);
        let result2 = mix_state(&state, event, entropy, delta_t);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_mix_state_avalanche() {
        let state = [0u8; 32];
        let event = b"sale_event";
        let entropy = b"entropy_source";
        let delta_t = 1000u64;

        let result1 = mix_state(&state, event, entropy, delta_t);
        
        let mut state2 = state;
        state2[0] ^= 1;
        let result2 = mix_state(&state2, event, entropy, delta_t);

        let mut diff_count = 0;
        for i in 0..32 {
            diff_count += (result1[i] ^ result2[i]).count_ones();
        }
        
        assert!(diff_count > 100, "Avalanche effect insufficient: {} bits changed", diff_count);
    }
}
