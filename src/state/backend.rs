use crate::{GasMeter, NetworkState, VMError};
use std::path::{Path, PathBuf};
use std::ops::{Deref, DerefMut};


/// Represents instance of a backed-up network state
pub struct Backend {
    network_state: NetworkState,
    path: PathBuf,
}

impl Backend {
    /// Creates new backend
    pub fn new<P>(path: P) -> Result<Self, VMError> where P: AsRef<Path> {
        let network_state = NetworkState::create_from_disk(path.as_ref()).map_err(|e| VMError::from(e))?;
        Ok(Backend{
            network_state,
            path: PathBuf::from(path.as_ref()),
        })
    }

    /// Persists the backend on disk
    pub fn persist(&mut self) -> Result<(), VMError> {
        let persistence_id = self.network_state
            .persist_to_store(self.store.clone())
            .expect("Error in persistence");

        let file_path = self.path.as_path().join(NetworkState::PERSISTENCE_ID_FILE_NAME);

        persistence_id.write(file_path)?;
        Ok(())
    }

    /// Restores the backend from disk
    pub fn restore<P>(path: P) -> Result<Self, VMError> where P: AsRef<Path> {
        let network_state = NetworkState::restore_from_disk(path.as_ref()).map_err( | e| VMError::from(e))?;
        Ok(Backend {
            network_state,
            path: PathBuf::from(path.as_ref()),
        })
    }

    /// Performs compacting of the backend
    pub fn compact<P>(from_path: P, to_path: P, gas: &mut GasMeter) -> Result<(), VMError> where P: AsRef<Path> {
        NetworkState::compact(from_path.as_ref(), to_path.as_ref(), gas).map_err( | e| VMError::from(e))?;
        Ok(())
    }
}

impl Deref for Backend {
    type Target = NetworkState;

    fn deref(&self) -> &Self::Target {
        &self.network_state
    }
}

impl DerefMut for Backend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.network_state
    }
}
