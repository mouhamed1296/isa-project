//! Frozen C ABI surface for MA-ISA.
//!
//! ## ABI STABILITY GUARANTEE
//!
//! All types and functions in this module are part of the stable C ABI.
//! They follow these rules:
//!
//! - All structs are `#[repr(C)]`
//! - All functions are `#[no_mangle]` with `extern "C"`
//! - No Rust-specific types cross the boundary
//! - Opaque handles are used for complex types
//! - No panics across FFI boundaries
//!
//! **ABI STABLE — DO NOT CHANGE WITHOUT MAJOR VERSION**

use crate::error::FfiError;
use crate::{register_runtime, get_runtime, remove_runtime};
use isa_runtime::{DeviceRuntime, FilePersistence};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

/// State vector containing all three axis states.
///
/// **ABI STABLE — DO NOT CHANGE WITHOUT MAJOR VERSION**
#[repr(C)]
pub struct StateVectorC {
    pub finance: [u8; 32],
    pub time: [u8; 32],
    pub hardware: [u8; 32],
}

/// Create a new runtime instance.
///
/// Returns an opaque handle (0 on failure).
///
/// **ABI STABLE — DO NOT CHANGE WITHOUT MAJOR VERSION**
#[no_mangle]
pub extern "C" fn isa_runtime_new(
    master_seed_ptr: *const u8,
    persistence_path: *const c_char,
) -> usize {
    if master_seed_ptr.is_null() || persistence_path.is_null() {
        return 0;
    }

    let master_seed = unsafe {
        let mut seed = [0u8; 32];
        std::ptr::copy_nonoverlapping(master_seed_ptr, seed.as_mut_ptr(), 32);
        seed
    };

    let path = unsafe {
        match CStr::from_ptr(persistence_path).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    let persistence = FilePersistence::new(path);
    let runtime = DeviceRuntime::new(master_seed, persistence);
    
    register_runtime(runtime)
}

#[no_mangle]
pub extern "C" fn isa_runtime_load_or_create(
    master_seed_ptr: *const u8,
    persistence_path: *const c_char,
) -> usize {
    if master_seed_ptr.is_null() || persistence_path.is_null() {
        return 0;
    }

    let master_seed = unsafe {
        let mut seed = [0u8; 32];
        std::ptr::copy_nonoverlapping(master_seed_ptr, seed.as_mut_ptr(), 32);
        seed
    };

    let path = unsafe {
        match CStr::from_ptr(persistence_path).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    let persistence = FilePersistence::new(path);
    match DeviceRuntime::load_or_create(master_seed, persistence) {
        Ok(runtime) => register_runtime(runtime),
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn isa_runtime_free(handle: usize) -> FfiError {
    if remove_runtime(handle) {
        FfiError::Success
    } else {
        FfiError::InvalidHandle
    }
}

/// Record a sale event and update state.
///
/// **ABI STABLE — DO NOT CHANGE WITHOUT MAJOR VERSION**
#[no_mangle]
pub extern "C" fn isa_record_sale(
    handle: usize,
    sale_ptr: *const u8,
    sale_len: usize,
    out_vector: *mut StateVectorC,
) -> FfiError {
    if sale_ptr.is_null() || out_vector.is_null() {
        return FfiError::NullPointer;
    }

    let sale_bytes = unsafe { slice::from_raw_parts(sale_ptr, sale_len) };

    let mut registry = match get_runtime(handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    let runtime = match registry.get_mut(&handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    match runtime.record_sale(sale_bytes) {
        Ok(vector) => {
            unsafe {
                (*out_vector).finance = vector.finance;
                (*out_vector).time = vector.time;
                (*out_vector).hardware = vector.hardware;
            }
            FfiError::Success
        }
        Err(e) => e.into(),
    }
}

#[no_mangle]
pub extern "C" fn isa_record_event(
    handle: usize,
    axis: u8,
    event_ptr: *const u8,
    event_len: usize,
    out_vector: *mut StateVectorC,
) -> FfiError {
    if event_ptr.is_null() || out_vector.is_null() {
        return FfiError::NullPointer;
    }

    let event_bytes = unsafe { slice::from_raw_parts(event_ptr, event_len) };

    let axis_enum = match axis {
        0 => isa_runtime::device::EventAxis::Finance,
        1 => isa_runtime::device::EventAxis::Time,
        2 => isa_runtime::device::EventAxis::Hardware,
        _ => return FfiError::InvalidState,
    };

    let mut registry = match get_runtime(handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    let runtime = match registry.get_mut(&handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    match runtime.record_event(axis_enum, event_bytes) {
        Ok(vector) => {
            unsafe {
                (*out_vector).finance = vector.finance;
                (*out_vector).time = vector.time;
                (*out_vector).hardware = vector.hardware;
            }
            FfiError::Success
        }
        Err(e) => e.into(),
    }
}

#[no_mangle]
pub extern "C" fn isa_save(handle: usize) -> FfiError {
    let mut registry = match get_runtime(handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    let runtime = match registry.get_mut(&handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    match runtime.save() {
        Ok(_) => FfiError::Success,
        Err(e) => e.into(),
    }
}

#[no_mangle]
pub extern "C" fn isa_get_state_vector(
    handle: usize,
    out_vector: *mut StateVectorC,
) -> FfiError {
    if out_vector.is_null() {
        return FfiError::NullPointer;
    }

    let registry = match get_runtime(handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    let runtime = match registry.get(&handle) {
        Some(r) => r,
        None => return FfiError::InvalidHandle,
    };

    let vector = runtime.state_vector();
    unsafe {
        (*out_vector).finance = vector.finance;
        (*out_vector).time = vector.time;
        (*out_vector).hardware = vector.hardware;
    }

    FfiError::Success
}

#[no_mangle]
pub extern "C" fn isa_axis_new(seed_ptr: *const u8) -> *mut u8 {
    if seed_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let seed = unsafe {
        let mut s = [0u8; 32];
        std::ptr::copy_nonoverlapping(seed_ptr, s.as_mut_ptr(), 32);
        s
    };

    let accumulator = isa_core::AxisAccumulator::new(seed);
    Box::into_raw(Box::new(accumulator)) as *mut u8
}

#[no_mangle]
pub extern "C" fn isa_axis_free(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut isa_core::AxisAccumulator);
        }
    }
}

/// Accumulate an event into a single axis.
///
/// **ABI STABLE — DO NOT CHANGE WITHOUT MAJOR VERSION**
#[no_mangle]
pub extern "C" fn isa_axis_accumulate(
    axis_ptr: *mut u8,
    event_ptr: *const u8,
    event_len: usize,
    entropy_ptr: *const u8,
    entropy_len: usize,
    delta_t: u64,
) -> FfiError {
    if axis_ptr.is_null() || event_ptr.is_null() || entropy_ptr.is_null() {
        return FfiError::NullPointer;
    }

    let accumulator = unsafe { &mut *(axis_ptr as *mut isa_core::AxisAccumulator) };
    let event = unsafe { slice::from_raw_parts(event_ptr, event_len) };
    let entropy = unsafe { slice::from_raw_parts(entropy_ptr, entropy_len) };

    accumulator.accumulate(event, entropy, delta_t);
    FfiError::Success
}

#[no_mangle]
pub extern "C" fn isa_axis_get_state(
    axis_ptr: *const u8,
    out_state: *mut u8,
) -> FfiError {
    if axis_ptr.is_null() || out_state.is_null() {
        return FfiError::NullPointer;
    }

    let accumulator = unsafe { &*(axis_ptr as *const isa_core::AxisAccumulator) };
    let state = accumulator.state();

    unsafe {
        std::ptr::copy_nonoverlapping(state.as_ptr(), out_state, 32);
    }

    FfiError::Success
}

#[no_mangle]
pub extern "C" fn isa_state_new(master_seed_ptr: *const u8) -> *mut u8 {
    if master_seed_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let master_seed = unsafe {
        let mut seed = [0u8; 32];
        std::ptr::copy_nonoverlapping(master_seed_ptr, seed.as_mut_ptr(), 32);
        seed
    };

    let state = isa_core::MultiAxisState::from_master_seed(master_seed);
    Box::into_raw(Box::new(state)) as *mut u8
}

#[no_mangle]
pub extern "C" fn isa_state_free(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut isa_core::MultiAxisState);
        }
    }
}

#[no_mangle]
pub extern "C" fn isa_get_version(
    major: *mut u16,
    minor: *mut u16,
    patch: *mut u16,
) -> FfiError {
    if major.is_null() || minor.is_null() || patch.is_null() {
        return FfiError::NullPointer;
    }

    unsafe {
        *major = isa_core::VERSION_MAJOR;
        *minor = isa_core::VERSION_MINOR;
        *patch = isa_core::VERSION_PATCH;
    }

    FfiError::Success
}
