use std::fmt;

/// FFI error codes.
///
/// **ABI STABLE — DO NOT CHANGE WITHOUT MAJOR VERSION**
///
/// Error codes are fixed integers. New errors must be added at the end.
/// Existing error codes must never be changed or removed.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiError {
    Success = 0,
    NullPointer = 1,
    InvalidHandle = 2,
    InvalidState = 3,
    EntropyFailed = 4,
    PersistenceFailed = 5,
    TimeFailed = 6,
    BufferTooSmall = 7,
    Unknown = 255,
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfiError::Success => write!(f, "Success"),
            FfiError::NullPointer => write!(f, "Null pointer provided"),
            FfiError::InvalidHandle => write!(f, "Invalid runtime handle"),
            FfiError::InvalidState => write!(f, "Invalid state"),
            FfiError::EntropyFailed => write!(f, "Entropy generation failed"),
            FfiError::PersistenceFailed => write!(f, "Persistence operation failed"),
            FfiError::TimeFailed => write!(f, "Time source failed"),
            FfiError::BufferTooSmall => write!(f, "Buffer too small"),
            FfiError::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl From<isa_runtime::RuntimeError> for FfiError {
    fn from(err: isa_runtime::RuntimeError) -> Self {
        match err {
            isa_runtime::RuntimeError::EntropyGenerationFailed => FfiError::EntropyFailed,
            isa_runtime::RuntimeError::PersistenceFailed(_) => FfiError::PersistenceFailed,
            isa_runtime::RuntimeError::TimeSourceFailed => FfiError::TimeFailed,
            isa_runtime::RuntimeError::InvalidState => FfiError::InvalidState,
        }
    }
}
