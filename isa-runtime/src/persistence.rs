use crate::{Result, RuntimeError};
use isa_core::MultiAxisState;
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

pub trait Persistence {
    fn save(&self, state: &MultiAxisState) -> Result<()>;
    fn load(&self) -> Result<MultiAxisState>;
    fn exists(&self) -> bool;
}

pub struct FilePersistence {
    path: PathBuf,
}

impl FilePersistence {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| RuntimeError::PersistenceFailed(e.to_string()))?;
        }
        Ok(())
    }
}

impl Persistence for FilePersistence {
    fn save(&self, state: &MultiAxisState) -> Result<()> {
        self.ensure_parent_dir()?;
        
        let bytes = state.to_bytes()
            .map_err(|e| RuntimeError::PersistenceFailed(e.to_string()))?;
        
        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, &bytes)
            .map_err(|e| RuntimeError::PersistenceFailed(e.to_string()))?;
        
        fs::rename(&temp_path, &self.path)
            .map_err(|e| RuntimeError::PersistenceFailed(e.to_string()))?;
        
        Ok(())
    }

    fn load(&self) -> Result<MultiAxisState> {
        let bytes = Zeroizing::new(
            fs::read(&self.path)
                .map_err(|e| RuntimeError::PersistenceFailed(e.to_string()))?
        );
        
        MultiAxisState::from_bytes(&bytes)
            .map_err(|_| RuntimeError::InvalidState)
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_persistence_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.bin");
        
        // Ensure clean state
        let _ = std::fs::remove_file(&state_path);
        
        let persistence = FilePersistence::new(&state_path);

        let master_seed = [1u8; 32];
        let state1 = MultiAxisState::from_master_seed(master_seed);

        assert!(!persistence.exists());
        persistence.save(&state1).unwrap();
        assert!(persistence.exists());

        let state2 = persistence.load().unwrap();
        assert_eq!(state1.state_vector(), state2.state_vector());
    }

    #[test]
    fn test_file_persistence_nested_dir() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("nested/dir/state.bin");
        let persistence = FilePersistence::new(&state_path);

        let master_seed = [1u8; 32];
        let state = MultiAxisState::from_master_seed(master_seed);

        persistence.save(&state).unwrap();
        assert!(persistence.exists());
    }
}
