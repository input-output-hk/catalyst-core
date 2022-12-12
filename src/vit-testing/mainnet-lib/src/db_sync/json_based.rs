use cardano_serialization_lib::metadata::GeneralTransactionMetadata;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use cardano_serialization_lib::Block;
use cardano_serialization_lib::utils::BigNum;
use serde::{Deserialize,Serialize};
use crate::db_sync::in_memory::{BlockNo, InMemoryDbSync};

/// Mock of real cardano db sync. At this moment we only stores transactions metadata
/// as the only purpose of existance for this struct is to provide catalyst voting registrations
/// Struct can be persisted and restored from json file using `serde_json`.
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonBasedDbSync {
    in_memory: InMemoryDbSync,
    pub(crate) db_file: PathBuf,
}

impl<'a> JsonBasedDbSync {

    /// Convert from `InMemoryDbSync`
    pub fn from_in_memory(in_memory: InMemoryDbSync, path: impl AsRef<Path>) -> JsonBasedDbSync {
        Self{
            in_memory,
            db_file: path.as_ref().to_path_buf()
        }
    }

    pub fn query_voting_transactions_with_bounds(&self, lower: Option<u64>, upper: Option<u64>) -> HashMap<BlockNo, Vec<GeneralTransactionMetadata>> {
        self.in_memory.query_voting_transactions_with_bounds(lower,upper)
    }

    pub fn stakes(&self) -> &HashMap<String, BigNum> {
        self.in_memory.stakes()
    }

    pub fn insert_block(
        &mut self,
        block: &Block,
    ){
        self.in_memory.on_block_propagation(block);
    }

    pub fn tx_metadata(&self) -> HashMap<BlockNo, Vec<GeneralTransactionMetadata>> {
        self.in_memory.metadata()
    }

    /// Creates new object with data
    #[must_use]
    pub fn new(db_file: impl AsRef<Path>) -> Self {
        Self {
            in_memory: Default::default(),
            db_file: db_file.as_ref().to_path_buf()
        }
    }

    /// Persists current state of db sync
    /// # Errors
    ///
    /// If cannot create file or cannot serialize to json
    pub fn persist(&self) -> Result<(), Error> {
        let mut file = File::create(&self.db_file)?;
        file.write_all(serde_json::to_string(&self)?.as_bytes())?;
        Ok(())
    }

    /// Restores current state of db sync from json file
    /// # Errors
    ///
    /// If file cannot be opened or cannot deserialize from json
    pub fn restore(path: impl AsRef<Path>) -> Result<Self, Error> {
        let db_sync_file = File::open(path)?;
        let db_sync: Self = serde_json::from_reader(db_sync_file)?;
        Ok(db_sync)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_file
    }
}

/// Db sync error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O related error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Serialization error
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}


#[cfg(test)]
mod tests {
    use assert_fs::prelude::PathChild;
    use crate::{MainnetNetworkBuilder, CardanoWallet, MainnetWalletStateBuilder};
    use assert_fs::TempDir;
    use crate::db_sync::json_based::JsonBasedDbSync;

    #[test]
    fn restore_persist_bijection_direct() {
        let testing_directory = TempDir::new().unwrap();
        let db = testing_directory.child("database.json");

        let alice = CardanoWallet::new(1_000);

        let (db_sync, _node,  _reps) = MainnetNetworkBuilder::default()
            .with(alice.as_direct_voter())
            .as_json(db);

        let before = db_sync.tx_metadata().clone();
        db_sync.persist().unwrap();
        let db_sync = JsonBasedDbSync::restore(&db_sync.db_file).unwrap();
        assert_eq!(before, db_sync.tx_metadata());
    }
}
