//! # isa-ffi
//!
//! C ABI and WASM bindings for MA-ISA.
//!
//! ## Invariants
//!
//! This crate is **ALLOWED** to:
//! - Use unsafe code (isolated to FFI boundaries only)
//! - Expose C-compatible types and functions
//! - Manage opaque handles and raw pointers
//!
//! This crate **MUST NOT**:
//! - Implement cryptographic logic (use isa-core)
//! - Implement platform logic (use isa-runtime)
//! - Expose Rust-specific types across the ABI
//! - Panic across FFI boundaries
//!
//! ## ABI Stability
//!
//! All `#[no_mangle]` functions and `#[repr(C)]` types are part of the stable ABI.
//! Changes to these require a MAJOR version bump.

pub mod c_api;
pub mod error;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use error::FfiError;

use isa_runtime::{DeviceRuntime, FilePersistence};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref RUNTIME_REGISTRY: Mutex<std::collections::HashMap<usize, Box<DeviceRuntime<FilePersistence>>>> = 
        Mutex::new(std::collections::HashMap::new());
}

static NEXT_HANDLE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

fn register_runtime(runtime: DeviceRuntime<FilePersistence>) -> usize {
    let handle = NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mut registry = RUNTIME_REGISTRY.lock().unwrap();
    registry.insert(handle, Box::new(runtime));
    handle
}

fn get_runtime(handle: usize) -> Option<std::sync::MutexGuard<'static, std::collections::HashMap<usize, Box<DeviceRuntime<FilePersistence>>>>> {
    let registry = RUNTIME_REGISTRY.lock().ok()?;
    if registry.contains_key(&handle) {
        Some(registry)
    } else {
        None
    }
}

fn remove_runtime(handle: usize) -> bool {
    let mut registry = RUNTIME_REGISTRY.lock().unwrap();
    registry.remove(&handle).is_some()
}
