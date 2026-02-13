//! # isa-runtime
//!
//! Platform-aware runtime for MA-ISA with entropy gathering, time sources, and persistence.
//!
//! ## Conformance Classification
//!
//! This crate contains a mix of conformance levels:
//!
//! - **NORMATIVE**: `policy` (threshold evaluation), `config` (configuration loading)
//! - **OPTIONAL**: `constraints`, `hierarchy`
//! - **EXPERIMENTAL**: `adaptive`
//! - **INFORMATIVE**: `device`, `entropy`, `persistence`, `time`, `profile`
//!
//! See individual module documentation for detailed conformance requirements.
//!
//! ## Scope
//!
//! This crate is **ALLOWED** to:
//! - Use system time sources
//! - Generate hardware entropy
//! - Perform file I/O for persistence
//! - Make platform-specific system calls
//!
//! This crate **SHALL NOT**:
//! - Implement cryptographic primitives (use isa-core)
//! - Expose language bindings (use isa-ffi)
//! - Violate determinism when given fixed inputs from tests

pub mod device;
pub mod entropy;
pub mod persistence;
pub mod time;
pub mod profile;
pub mod policy;
pub mod constraints;
pub mod hierarchy;
pub mod adaptive;
pub mod config;

pub use device::{DeviceRuntime, EventAxis, RecoveryAudit};
pub use entropy::EntropySource;
pub use persistence::{Persistence, FilePersistence};
pub use time::MonotonicClock;
pub use profile::{DimensionProfile, DimensionMapping, standard_maisa_profile};
pub use policy::{DimensionPolicy, PolicySet, RecoveryStrategy};
pub use constraints::{DimensionConstraint, ConstraintSet, ConstraintType};
pub use hierarchy::{DimensionNode, DimensionHierarchy, DimensionMetadata};
pub use adaptive::{AdaptiveProfile, DimensionObservation, DimensionStats, MLModel, ModelContext, ModelMetadata};
pub use config::{IsaConfig, GlobalConfig, DimensionConfig, ConstraintConfig, HierarchyConfig, load_from_env};

pub type Result<T> = core::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    EntropyGenerationFailed,
    PersistenceFailed(String),
    TimeSourceFailed,
    InvalidState,
}

impl core::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RuntimeError::EntropyGenerationFailed => write!(f, "Failed to generate entropy"),
            RuntimeError::PersistenceFailed(msg) => write!(f, "Persistence error: {}", msg),
            RuntimeError::TimeSourceFailed => write!(f, "Time source unavailable"),
            RuntimeError::InvalidState => write!(f, "Invalid state"),
        }
    }
}

impl std::error::Error for RuntimeError {}
