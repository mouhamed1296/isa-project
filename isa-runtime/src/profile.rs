//! Domain-specific profiles for integrity dimensions.
//!
//! This module defines how domain semantics (finance, time, hardware, etc.)
//! map to the domain-agnostic dimension indices in isa-core.
//!
//! ## Design
//!
//! - Domain semantics live ONLY in isa-runtime
//! - isa-core remains completely domain-agnostic
//! - Profiles are configuration-driven and extensible

/// Dimension profile defining semantic meaning for integrity dimensions.
///
/// This maps domain concepts to dimension indices without polluting
/// the core cryptographic layer with domain-specific logic.
#[derive(Debug, Clone)]
pub struct DimensionProfile {
    pub dimension_count: usize,
    pub mappings: Vec<DimensionMapping>,
}

/// Mapping from a semantic label to a dimension index.
#[derive(Debug, Clone)]
pub struct DimensionMapping {
    pub label: &'static str,
    pub index: usize,
    pub description: &'static str,
}

impl DimensionProfile {
    /// Get the dimension index for a given semantic label.
    pub fn index_for(&self, label: &str) -> Option<usize> {
        self.mappings
            .iter()
            .find(|m| m.label == label)
            .map(|m| m.index)
    }

    /// Get the semantic label for a given dimension index.
    pub fn label_for(&self, index: usize) -> Option<&'static str> {
        self.mappings
            .iter()
            .find(|m| m.index == index)
            .map(|m| m.label)
    }
}

/// Standard 3-axis MA-ISA profile (finance, time, hardware).
///
/// This is the original domain configuration for MA-ISA.
/// Dimension 0: Financial transactions and monetary events
/// Dimension 1: Temporal progression and ordering
/// Dimension 2: Hardware-specific entropy and device identity
pub fn standard_maisa_profile() -> DimensionProfile {
    DimensionProfile {
        dimension_count: 3,
        mappings: vec![
            DimensionMapping {
                label: "finance",
                index: 0,
                description: "Financial transactions and monetary events",
            },
            DimensionMapping {
                label: "time",
                index: 1,
                description: "Temporal progression and ordering",
            },
            DimensionMapping {
                label: "hardware",
                index: 2,
                description: "Hardware-specific entropy and device identity",
            },
        ],
    }
}

/// Dimension index constants for the standard MA-ISA profile.
///
/// These provide convenient access to dimension indices without
/// string lookups, while keeping semantics in the runtime layer.
pub mod standard_indices {
    pub const FINANCE: usize = 0;
    pub const TIME: usize = 1;
    pub const HARDWARE: usize = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_profile() {
        let profile = standard_maisa_profile();
        
        assert_eq!(profile.dimension_count, 3);
        assert_eq!(profile.index_for("finance"), Some(0));
        assert_eq!(profile.index_for("time"), Some(1));
        assert_eq!(profile.index_for("hardware"), Some(2));
        assert_eq!(profile.index_for("unknown"), None);
    }

    #[test]
    fn test_reverse_lookup() {
        let profile = standard_maisa_profile();
        
        assert_eq!(profile.label_for(0), Some("finance"));
        assert_eq!(profile.label_for(1), Some("time"));
        assert_eq!(profile.label_for(2), Some("hardware"));
        assert_eq!(profile.label_for(3), None);
    }

    #[test]
    fn test_standard_indices() {
        use standard_indices::*;
        
        assert_eq!(FINANCE, 0);
        assert_eq!(TIME, 1);
        assert_eq!(HARDWARE, 2);
    }
}
