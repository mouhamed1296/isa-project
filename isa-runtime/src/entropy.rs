use crate::{Result, RuntimeError};
use zeroize::Zeroizing;

pub struct EntropySource;

impl EntropySource {
    pub fn new() -> Self {
        Self
    }

    pub fn gather(&self, size: usize) -> Result<Zeroizing<Vec<u8>>> {
        let mut buffer = vec![0u8; size];
        getrandom::getrandom(&mut buffer)
            .map_err(|_| RuntimeError::EntropyGenerationFailed)?;
        Ok(Zeroizing::new(buffer))
    }

    pub fn gather_32(&self) -> Result<[u8; 32]> {
        let mut buffer = [0u8; 32];
        getrandom::getrandom(&mut buffer)
            .map_err(|_| RuntimeError::EntropyGenerationFailed)?;
        Ok(buffer)
    }
}

impl Default for EntropySource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_generation() {
        let source = EntropySource::new();
        let entropy1 = source.gather(32).unwrap();
        let entropy2 = source.gather(32).unwrap();
        
        assert_eq!(entropy1.len(), 32);
        assert_eq!(entropy2.len(), 32);
        assert_ne!(&*entropy1, &*entropy2);
    }

    #[test]
    fn test_entropy_32() {
        let source = EntropySource::new();
        let entropy1 = source.gather_32().unwrap();
        let entropy2 = source.gather_32().unwrap();
        
        assert_ne!(entropy1, entropy2);
    }
}
