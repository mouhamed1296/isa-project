#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn current() -> Self {
        Self::new(
            crate::VERSION_MAJOR,
            crate::VERSION_MINOR,
            crate::VERSION_PATCH,
        )
    }

    pub const fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        let mut bytes = [0u8; 6];
        bytes[0..2].copy_from_slice(&self.major.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.minor.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.patch.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        Self {
            major: u16::from_le_bytes([bytes[0], bytes[1]]),
            minor: u16::from_le_bytes([bytes[2], bytes[3]]),
            patch: u16::from_le_bytes([bytes[4], bytes[5]]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_serialization() {
        let v = Version::new(1, 2, 3);
        let bytes = v.to_bytes();
        let v2 = Version::from_bytes(&bytes);
        assert_eq!(v, v2);
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
    }
}
