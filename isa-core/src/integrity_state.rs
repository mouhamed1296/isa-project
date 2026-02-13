//! Multi-dimensional integrity state coordinator.
//!
//! This module provides a domain-agnostic container for N independent integrity dimensions.
//! It replaces the domain-specific `MultiAxisState` with a neutral, indexed model.
//!
//! ## Mathematical Model
//!
//! The integrity state is a vector S⃗ ∈ Z^N_{2^256} where:
//! - N is the number of dimensions (compile-time constant)
//! - Each dimension evolves independently according to Equation 2
//! - Divergence is computed element-wise using circular distance
//!
//! ## Design
//!
//! - Uses const generics for compile-time dimension count
//! - Dimensions are accessed by numeric index (0..N)
//! - No domain semantics in type system or API
//! - Backward compatible via type aliases

use crate::dimension::DimensionAccumulator;
use crate::divergence::CircularDistance;
use crate::version::Version;
use crate::STATE_SIZE;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(all(feature = "serde", not(feature = "std")))]
extern crate alloc;

#[cfg(all(feature = "serde", not(feature = "std")))]
use alloc::vec::Vec;

/// Opaque dimension identifier.
///
/// This is a fixed-size byte array with no semantic meaning at the core layer.
/// Used for KDF domain separation without encoding domain-specific information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionId([u8; 16]);

impl DimensionId {
    /// Create a dimension ID from a numeric index.
    ///
    /// The index is encoded as a little-endian u128 to create an opaque identifier.
    /// This provides unique separation without semantic meaning.
    pub fn from_index(index: usize) -> Self {
        let mut id = [0u8; 16];
        id[..8].copy_from_slice(&(index as u64).to_le_bytes());
        Self(id)
    }

    /// Create a dimension ID from raw bytes.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of this dimension ID.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Construct KDF label: fixed prefix || dimension_id
    ///
    /// Format: b"isa.dim" || dimension_id_bytes
    /// This provides domain separation without human-readable semantics.
    pub fn to_kdf_label(&self) -> [u8; 23] {
        const PREFIX: &[u8] = b"isa.dim";
        let mut label = [0u8; 23];
        label[..7].copy_from_slice(PREFIX);
        label[7..].copy_from_slice(&self.0);
        label
    }
}

/// Multi-dimensional integrity state with N independent dimensions.
///
/// This is the domain-agnostic replacement for `MultiAxisState`.
/// Dimensions are indexed 0..N and have no semantic meaning at this layer.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct IntegrityState<const N: usize> {
    dimensions: [DimensionAccumulator; N],
    #[zeroize(skip)]
    version: Version,
}

// Serde implementation for IntegrityState<3>
// Bincode works better with derive than manual implementation
#[cfg(feature = "serde")]
impl Serialize for IntegrityState<3> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tuple = serializer.serialize_tuple(4)?;
        tuple.serialize_element(&self.dimensions[0])?;
        tuple.serialize_element(&self.dimensions[1])?;
        tuple.serialize_element(&self.dimensions[2])?;
        tuple.serialize_element(&self.version)?;
        tuple.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for IntegrityState<3> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor, SeqAccess};
        
        struct IntegrityStateVisitor;
        
        impl<'de> Visitor<'de> for IntegrityStateVisitor {
            type Value = IntegrityState<3>;
            
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("IntegrityState<3> as tuple")
            }
            
            fn visit_seq<V>(self, mut seq: V) -> Result<IntegrityState<3>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let dim0 = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let dim1 = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let dim2 = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let version = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                
                Ok(IntegrityState {
                    dimensions: [dim0, dim1, dim2],
                    version,
                })
            }
        }
        
        deserializer.deserialize_tuple(4, IntegrityStateVisitor)
    }
}

impl<const N: usize> IntegrityState<N> {
    /// Create a new integrity state from per-dimension seeds.
    pub fn new(seeds: [[u8; STATE_SIZE]; N]) -> Self {
        let dimensions = seeds.map(|seed| DimensionAccumulator::new(seed));
        Self {
            dimensions,
            version: Version::current(),
        }
    }

    /// Create integrity state from a master seed using KDF.
    ///
    /// Each dimension gets a unique seed derived from:
    /// seed_i = KDF(b"isa.dim" || dimension_id, master_seed)
    ///
    /// Where dimension_id is an opaque 16-byte identifier with no semantic meaning.
    pub fn from_master_seed(master_seed: [u8; STATE_SIZE]) -> Self {
        use crate::kdf::Kdf;

        let mut dimensions = core::array::from_fn(|_| DimensionAccumulator::new([0u8; STATE_SIZE]));
        
        for i in 0..N {
            let dimension_id = DimensionId::from_index(i);
            let seed = Kdf::derive_key(&dimension_id.to_kdf_label(), &[&master_seed]);
            dimensions[i] = DimensionAccumulator::new(seed);
        }

        Self {
            dimensions,
            version: Version::current(),
        }
    }

    /// Get a reference to a specific dimension by index.
    pub fn dimension(&self, index: usize) -> Option<&DimensionAccumulator> {
        self.dimensions.get(index)
    }

    /// Get a mutable reference to a specific dimension by index.
    pub fn dimension_mut(&mut self, index: usize) -> Option<&mut DimensionAccumulator> {
        self.dimensions.get_mut(index)
    }

    /// Get the version of this state.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Extract the state vector (all dimension states).
    pub fn state_vector(&self) -> DimensionVector<N> {
        let mut values = [[0u8; STATE_SIZE]; N];
        for (i, dim) in self.dimensions.iter().enumerate() {
            values[i] = dim.state();
        }
        DimensionVector { values }
    }

    /// Calculate divergence between this state and another.
    ///
    /// Returns the circular distance for each dimension independently.
    pub fn divergence(&self, other: &Self) -> DivergenceVector<N> {
        let mut values = [[0u8; STATE_SIZE]; N];
        for i in 0..N {
            values[i] = CircularDistance::min_distance(
                &self.dimensions[i].state(),
                &other.dimensions[i].state(),
            );
        }
        DivergenceVector { values }
    }
}

impl<const N: usize> core::fmt::Debug for IntegrityState<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IntegrityState")
            .field("dimension_count", &N)
            .field("dimensions", &self.dimensions)
            .field("version", &self.version)
            .finish()
    }
}

// Serialization methods for IntegrityState<3>
impl IntegrityState<3> {
    #[cfg(all(feature = "serde", feature = "std"))]
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        let versioned = VersionedIntegrityState {
            version: self.version,
            state: self.clone(),
        };
        bincode::serialize(&versioned)
    }

    #[cfg(all(feature = "serde", not(feature = "std")))]
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        let versioned = VersionedIntegrityState {
            version: self.version,
            state: self.clone(),
        };
        bincode::serialize(&versioned)
    }

    #[cfg(feature = "serde")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IntegrityStateError> {
        let versioned: VersionedIntegrityState = bincode::deserialize(bytes)
            .map_err(|_| IntegrityStateError::DeserializationFailed)?;

        if !versioned.version.is_compatible(&Version::current()) {
            return Err(IntegrityStateError::IncompatibleVersion {
                found: versioned.version,
                expected: Version::current(),
            });
        }

        Ok(versioned.state)
    }
}

/// Vector of dimension states.
///
/// This is the domain-agnostic replacement for `StateVector`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionVector<const N: usize> {
    pub values: [[u8; STATE_SIZE]; N],
}

#[cfg(feature = "serde")]
impl Serialize for DimensionVector<3> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DimensionVector", 3)?;
        state.serialize_field("v0", &self.values[0])?;
        state.serialize_field("v1", &self.values[1])?;
        state.serialize_field("v2", &self.values[2])?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for DimensionVector<3> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor, MapAccess};
        
        struct DimensionVectorVisitor;
        
        impl<'de> Visitor<'de> for DimensionVectorVisitor {
            type Value = DimensionVector<3>;
            
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("struct DimensionVector<3>")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<DimensionVector<3>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut v0 = None;
                let mut v1 = None;
                let mut v2 = None;
                
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "v0" => v0 = Some(map.next_value()?),
                        "v1" => v1 = Some(map.next_value()?),
                        "v2" => v2 = Some(map.next_value()?),
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                
                Ok(DimensionVector {
                    values: [
                        v0.ok_or_else(|| de::Error::missing_field("v0"))?,
                        v1.ok_or_else(|| de::Error::missing_field("v1"))?,
                        v2.ok_or_else(|| de::Error::missing_field("v2"))?,
                    ],
                })
            }
        }
        
        const FIELDS: &[&str] = &["v0", "v1", "v2"];
        deserializer.deserialize_struct("DimensionVector", FIELDS, DimensionVectorVisitor)
    }
}

impl<const N: usize> DimensionVector<N> {
    /// Get the state for a specific dimension.
    pub fn get(&self, index: usize) -> Option<&[u8; STATE_SIZE]> {
        self.values.get(index)
    }
}

/// Vector of divergence values (one per dimension).
///
/// This is the domain-agnostic replacement for `DivergenceMetric`.
#[derive(Debug, Clone, Copy)]
pub struct DivergenceVector<const N: usize> {
    pub values: [[u8; STATE_SIZE]; N],
}

impl<const N: usize> DivergenceVector<N> {
    /// Get the divergence for a specific dimension.
    pub fn get(&self, index: usize) -> Option<&[u8; STATE_SIZE]> {
        self.values.get(index)
    }
}

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct VersionedIntegrityState {
    version: Version,
    state: IntegrityState<3>,
}

#[derive(Debug)]
pub enum IntegrityStateError {
    DeserializationFailed,
    IncompatibleVersion { found: Version, expected: Version },
}

impl core::fmt::Display for IntegrityStateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IntegrityStateError::DeserializationFailed => write!(f, "Failed to deserialize integrity state"),
            IntegrityStateError::IncompatibleVersion { found, expected } => {
                write!(
                    f,
                    "Incompatible version: found {}.{}.{}, expected {}.{}.{}",
                    found.major, found.minor, found.patch,
                    expected.major, expected.minor, expected.patch
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IntegrityStateError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Normative test: 3-dimensional state (matches original MA-ISA)
    #[test]
    fn test_three_dimension_state_creation() {
        let master_seed = [1u8; 32];
        let state: IntegrityState<3> = IntegrityState::from_master_seed(master_seed);
        
        assert_eq!(state.version(), Version::current());
        assert!(state.dimension(0).is_some());
        assert!(state.dimension(1).is_some());
        assert!(state.dimension(2).is_some());
        assert!(state.dimension(3).is_none());
    }

    #[test]
    fn test_state_vector_extraction() {
        let master_seed = [1u8; 32];
        let state: IntegrityState<3> = IntegrityState::from_master_seed(master_seed);
        let vector = state.state_vector();
        
        assert_eq!(vector.values[0], state.dimension(0).unwrap().state());
        assert_eq!(vector.values[1], state.dimension(1).unwrap().state());
        assert_eq!(vector.values[2], state.dimension(2).unwrap().state());
    }

    #[test]
    fn test_divergence_zero() {
        let master_seed = [1u8; 32];
        let state1: IntegrityState<3> = IntegrityState::from_master_seed(master_seed);
        let state2: IntegrityState<3> = IntegrityState::from_master_seed(master_seed);
        
        let div = state1.divergence(&state2);
        assert_eq!(div.values[0], [0u8; 32]);
        assert_eq!(div.values[1], [0u8; 32]);
        assert_eq!(div.values[2], [0u8; 32]);
    }

    #[test]
    fn test_divergence_nonzero() {
        let master_seed1 = [1u8; 32];
        let master_seed2 = [2u8; 32];
        let state1: IntegrityState<3> = IntegrityState::from_master_seed(master_seed1);
        let state2: IntegrityState<3> = IntegrityState::from_master_seed(master_seed2);
        
        let div = state1.divergence(&state2);
        assert_ne!(div.values[0], [0u8; 32]);
        assert_ne!(div.values[1], [0u8; 32]);
        assert_ne!(div.values[2], [0u8; 32]);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialization_roundtrip() {
        let master_seed = [1u8; 32];
        let state1: IntegrityState<3> = IntegrityState::from_master_seed(master_seed);
        
        let bytes = state1.to_bytes().unwrap();
        eprintln!("Serialized {} bytes", bytes.len());
        
        let state2 = match IntegrityState::<3>::from_bytes(&bytes) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Deserialization failed: {:?}", e);
                eprintln!("First 100 bytes: {:?}", &bytes[..bytes.len().min(100)]);
                panic!("Failed to deserialize");
            }
        };
        
        assert_eq!(state1.state_vector(), state2.state_vector());
        assert_eq!(state1.version(), state2.version());
    }

    // Test with different dimension counts
    #[test]
    fn test_variable_dimension_count() {
        let master_seed = [1u8; 32];
        
        let state2: IntegrityState<2> = IntegrityState::from_master_seed(master_seed);
        assert!(state2.dimension(0).is_some());
        assert!(state2.dimension(1).is_some());
        assert!(state2.dimension(2).is_none());
        
        let state5: IntegrityState<5> = IntegrityState::from_master_seed(master_seed);
        assert!(state5.dimension(4).is_some());
        assert!(state5.dimension(5).is_none());
    }
}
