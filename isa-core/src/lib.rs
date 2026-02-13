//! # isa-core
//!
//! Pure cryptographic implementation of Multi-Axis Integral State Accumulator (MA-ISA).
//!
//! ## Conformance Classification
//!
//! **This module defines NORMATIVE behavior required for conformance with the MA-ISA
//! integrity model.** All types and functions in this module SHALL be implemented
//! by conforming implementations.
//!
//! ## Scope
//!
//! This module provides:
//! - Deterministic integrity dimension accumulation
//! - Multi-dimensional state management
//! - Divergence calculation between states
//! - Runtime-configurable dimension support
//!
//! ## Invariants
//!
//! This crate SHALL remain deterministic, platform-independent, and free of side effects.
//! Any use of time, randomness, IO, or system state is **FORBIDDEN**.
//!
//! All operations SHALL be:
//! - Deterministic (same inputs → same outputs)
//! - Pure (no side effects)
//! - Platform-independent (no OS-specific code)
//! - no_std compatible

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod axis;
pub mod divergence;
pub mod kdf;
pub mod version;
pub mod dimension;
pub mod integrity_state;
pub mod compat;
pub mod dynamic;

#[cfg(kani)]
pub mod verify;

// Domain-agnostic core API
pub use axis::AxisAccumulator;
pub use dimension::DimensionAccumulator;
pub use integrity_state::{IntegrityState, DimensionVector, DivergenceVector, IntegrityStateError, DimensionId};
pub use divergence::CircularDistance;
pub use version::Version;
pub use dynamic::DynamicIntegrityState;

// Backward-compatible domain-specific API
pub use compat::{MultiAxisState, StateVector, DivergenceMetric, StateError, MultiAxisStateExt};

pub const STATE_SIZE: usize = 32;
pub const VERSION_MAJOR: u16 = 0;
pub const VERSION_MINOR: u16 = 1;
pub const VERSION_PATCH: u16 = 0;
