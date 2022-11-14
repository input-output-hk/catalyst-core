use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::metadata::GeneralTransactionMetadata;
use jormungandr_lib::interfaces::BlockDate;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const CARDANO_MAINNET_SLOTS_PER_EPOCH: u64 = 43200;

/// Mock of real cardano db sync. At this moment we only stores transactions metadata
/// as the only purpose of existance for this struct is to provide catalyst voting registrations
/// Struct can be persisted and restored from json file using `serde_json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct InMemoryDbSync {
    tx_metadata: HashMap<BlockDate, Vec<GeneralTransactionMetadata>>,
    stakes: HashMap<String, u64>,
    settings: Settings,
    ///Fake db as standard json file
    db: PathBuf,
}

impl InMemoryDbSync {
    /// Creates new object
    #[must_use]
    pub fn new(temp_dir: &TempDir) -> Self {
        Self {
            tx_metadata: HashMap::new(),
            stakes: HashMap::new(),
            settings: Settings::default(),
            db: temp_dir.child("db_sync.db").path().to_path_buf(),
        }
    }

    /// Path to fake db = json file
    #[must_use]
    pub fn db_path(&self) -> &Path {
        &self.db
    }

    /// Accept new transaction metadata
    pub fn push_transaction(
        &mut self,
        block_date: BlockDate,
        registration: GeneralTransactionMetadata,
    ) {
        match self.tx_metadata.entry(block_date) {
            Entry::Vacant(e) => {
                e.insert(vec![registration]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(registration);
            }
        }
    }

    /// Push new wallet stake information
    pub fn push_address(&mut self, address: &Address, stake: u64) {
        self.stakes.insert(address.to_hex(), stake);
    }

    /// Gets all transactions metadata without bounds
    #[must_use]
    pub fn query_all_registration_transactions(&self) -> Vec<GeneralTransactionMetadata> {
        self.tx_metadata
            .values()
            .cloned()
            .fold(vec![], |mut vec, mut value| {
                vec.append(&mut value);
                vec
            })
    }

    /// Gets all transactions metadata with `slot_no` upper and lower bounds
    pub fn query_voting_transactions_with_bounds(
        &self,
        lower: Option<u64>,
        upper: Option<u64>,
    ) -> HashMap<&BlockDate, &Vec<GeneralTransactionMetadata>> {
        let maybe_lower_block_date = lower.map(BlockDate::from_absolute_slot_no);
        let maybe_upper_block_date = upper.map(BlockDate::from_absolute_slot_no);

        self.tx_metadata
            .iter()
            .filter(|(block, _)| {
                if let Some(result) = maybe_lower_block_date.map(|x| x > **block) {
                    if result {
                        return false;
                    }
                }
                if let Some(result) = maybe_upper_block_date.map(|x| x < **block) {
                    if result {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// gets reference to db sync connection settings
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// gets all known to dbsync wallet ada distribution
    #[must_use]
    pub fn stakes(&self) -> &HashMap<String, u64> {
        &self.stakes
    }

    /// Persists current state of db sync
    /// # Errors
    ///
    /// If cannot create file or cannot serialize to json
    pub fn persist(&self) -> Result<(), Error> {
        let mut file = std::fs::File::create(&self.db)?;
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub db_name: String,
    pub db_user: String,
    pub db_host: String,
    pub db_pass: String,
}

pub trait BlockDateFromCardanoAbsoluteSlotNo {
    fn from_absolute_slot_no(absolute_slot_no: u64) -> Self;
}

impl BlockDateFromCardanoAbsoluteSlotNo for BlockDate {
    fn from_absolute_slot_no(absolute_slot_no: u64) -> Self {
        let epoch = absolute_slot_no / CARDANO_MAINNET_SLOTS_PER_EPOCH;
        let slot = absolute_slot_no - epoch * CARDANO_MAINNET_SLOTS_PER_EPOCH;
        BlockDate::new(u32::try_from(epoch).unwrap(), u32::try_from(slot).unwrap())
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
    use assert_fs::TempDir;
    use crate::{InMemoryDbSync, MainnetNetworkBuilder, MainnetWallet, MainnetWalletStateBuilder};

    #[test]
    fn restore_persist_bijection_direct() {
        let testing_directory = TempDir::new().unwrap();

        let alice = MainnetWallet::new(1_000);

        let (db_sync, _reps) = MainnetNetworkBuilder::default()
            .with(alice.as_direct_voter())
            .build(&testing_directory);

        let before = db_sync.tx_metadata.clone();
        db_sync.persist().unwrap();
        let db_sync = InMemoryDbSync::restore(&db_sync.db).unwrap();
        assert_eq!(before,db_sync.tx_metadata);
    }
}