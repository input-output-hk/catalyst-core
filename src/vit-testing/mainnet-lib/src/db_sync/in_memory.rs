use crate::{Block0, InMemoryNode, CARDANO_MAINNET_SLOTS_PER_EPOCH};
use cardano_serialization_lib::{BigNum, GeneralTransactionMetadata};
use cardano_serialization_lib::{Block, Transaction, TransactionWitnessSet};
use futures::executor::block_on;
use futures_util::StreamExt;
use jormungandr_lib::interfaces::BlockDate;
use pharos::{Channel, Observable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;

pub type BlockNo = u32;
pub type Address = String;

/// thread safe `InMemoryDbSync`. It has inner struct `db_sync` with rw lock guard and handle to update
/// thread which listen to `InMemoryNode` mock block updates
pub struct SharedInMemoryDbSync {
    pub(crate) update_thread: JoinHandle<()>,
    // Allowing for now since there is no usage yet in explorer service
    #[allow(dead_code)]
    pub(crate) db_sync: Arc<RwLock<InMemoryDbSync>>,
}

impl Drop for SharedInMemoryDbSync {
    fn drop(&mut self) {
        self.update_thread.abort();
    }
}

/// Mock of real cardano db sync. At this moment we only stores transactions metadata
/// as the only purpose of existance for this struct is to provide catalyst voting registrations
/// Struct can be persisted and restored from json file using `serde_json`.
#[derive(Serialize, Deserialize, Default)]
pub struct InMemoryDbSync {
    pub(crate) transactions: HashMap<BlockNo, Vec<Transaction>>,
    pub(crate) blocks: Vec<Block>,
    stakes: HashMap<Address, BigNum>,
    settings: Settings,
}

impl Debug for InMemoryDbSync {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.settings)
    }
}

impl InMemoryDbSync {
    /// Creates new instance out of block0
    #[must_use]
    pub fn from_block0(block0: &Block0) -> Self {
        let mut db_sync = InMemoryDbSync::default();
        db_sync.on_block_propagation(&block0.block);
        db_sync
    }

    /// Create an empty instance
    #[must_use]
    pub fn empty() -> Self {
        InMemoryDbSync {
            transactions: HashMap::new(),
            blocks: vec![],
            stakes: HashMap::new(),
            settings: Settings::default(),
        }
    }

    /// Connects to Cardano mock node using simple observer/observable mechanism
    ///
    /// # Panics
    ///
    /// On accessing db sync state
    pub fn connect_to_node(self, node: &mut InMemoryNode) -> SharedInMemoryDbSync {
        let mut observer = block_on(async {
            node.observe(Channel::Bounded(1).into())
                .await
                .expect("observer")
        });

        let shared_db_sync = Arc::new(RwLock::new(self));
        let db_sync = shared_db_sync.clone();

        let handle = tokio::spawn(async move {
            loop {
                let block = observer.next().await;
                if let Some(block) = block {
                    db_sync.write().unwrap().on_block_propagation(&block);
                }
            }
        });

        SharedInMemoryDbSync {
            update_thread: handle,
            db_sync: shared_db_sync,
        }
    }

    /// Retrieves db sync content as string
    ///
    /// # Errors
    ///
    /// On deserialization issues
    pub fn try_as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    /// Accept new block
    ///
    /// # Panics
    ///
    /// On integer overflow
    pub fn on_block_propagation(&mut self, block: &Block) {
        self.blocks.push(block.clone());

        let mut transactions = vec![];
        let bodies = block.transaction_bodies();
        for i in 0..bodies.len() {
            let outputs = bodies.get(i).outputs();

            for i in 0..outputs.len() {
                let output = outputs.get(i);
                let stake = output.amount().coin();
                self.stakes
                    .entry(output.address().to_hex())
                    .and_modify(|x| *x = x.checked_add(&stake).unwrap())
                    .or_insert(stake);
            }

            transactions.push(Transaction::new(
                &bodies.get(i),
                &TransactionWitnessSet::new(),
                block.auxiliary_data_set().get(u32::try_from(i).unwrap()),
            ));
        }

        self.transactions
            .insert(block.header().header_body().block_number(), transactions);
    }

    /// Query transaction by it's hash representation
    #[must_use]
    pub fn transaction_by_hash(&self, hash: &str) -> Vec<(Option<&Block>, &Transaction)> {
        self.transactions
            .iter()
            .filter_map(|(block, txs)| {
                if let Some(tx) = txs.iter().find(|tx| tx.to_hex() == hash) {
                    let block = self
                        .blocks
                        .iter()
                        .find(|x| x.header().header_body().block_number() == *block);

                    return Some((block, tx));
                }
                None
            })
            .collect()
    }

    /// Gets all transactions metadata without bounds
    #[must_use]
    pub fn query_all_registration_transactions(&self) -> Vec<GeneralTransactionMetadata> {
        self.metadata()
            .values()
            .cloned()
            .fold(vec![], |mut vec, mut value| {
                vec.append(&mut value);
                vec
            })
    }

    /// Gets all metadata per block number
    #[must_use]
    pub fn metadata(&self) -> HashMap<BlockNo, Vec<GeneralTransactionMetadata>> {
        self.transactions
            .iter()
            .map(|(block, tx)| {
                let metadata = tx
                    .iter()
                    .filter_map(|x| {
                        if let Some(auxiliary_data) = x.auxiliary_data() {
                            if let Some(metadata) = auxiliary_data.metadata() {
                                return Some(metadata);
                            }
                        }
                        None
                    })
                    .collect();
                (*block, metadata)
            })
            .collect()
    }

    /// Gets all transactions metadata with `slot_no` upper and lower bounds
    #[must_use]
    pub fn query_voting_transactions_with_bounds(
        &self,
        lower: u64,
        upper: u64,
    ) -> HashMap<BlockNo, Vec<GeneralTransactionMetadata>> {
        self.metadata()
            .into_iter()
            .filter(|(block_no, _)| lower <= u64::from(*block_no) && u64::from(*block_no) <= upper)
            .collect()
    }

    /// gets reference to db sync connection settings
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// gets all known to dbsync wallet ada distribution
    #[must_use]
    pub fn stakes(&self) -> &HashMap<String, BigNum> {
        &self.stakes
    }

    /// Persists current state of db sync
    /// # Errors
    ///
    /// If cannot create file or cannot serialize to json
    pub fn persist(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut file = File::create(path)?;
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
    pub name: String,
    pub user: String,
    pub host: String,
    pub pass: String,
}

/// Basic converter from absolute slot number and {epoch,slot} pair
pub trait BlockDateFromCardanoAbsoluteSlotNo {
    /// Converts absolute slot number to block date
    fn from_absolute_slot_no(absolute_slot_no: u64) -> Self;
    /// Converts epoch/slot representation to absolute slot number
    fn to_absolute_slot_no(self) -> u64;
}

impl BlockDateFromCardanoAbsoluteSlotNo for BlockDate {
    fn from_absolute_slot_no(absolute_slot_no: u64) -> Self {
        let epoch = absolute_slot_no / CARDANO_MAINNET_SLOTS_PER_EPOCH;
        let slot = absolute_slot_no - epoch * CARDANO_MAINNET_SLOTS_PER_EPOCH;
        BlockDate::new(u32::try_from(epoch).unwrap(), u32::try_from(slot).unwrap())
    }

    fn to_absolute_slot_no(self) -> u64 {
        u64::from(
            self.epoch() * (u32::try_from(CARDANO_MAINNET_SLOTS_PER_EPOCH).unwrap()) + self.slot(),
        )
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
    use crate::network::wallet_state::MainnetWalletStateBuilder;
    use crate::network::MainnetNetworkBuilder;
    use crate::{Block0, InMemoryNode};
    use crate::{CardanoWallet, InMemoryDbSync};
    use assert_fs::fixture::PathChild;
    use assert_fs::TempDir;
    use cardano_serialization_lib::BigNum;
    use std::time::Duration;

    #[tokio::test]
    async fn restore_persist_bijection_direct() {
        let testing_directory = TempDir::new().unwrap();

        let alice = CardanoWallet::new(1_000);

        let (db_sync, _node, _reps) = MainnetNetworkBuilder::default()
            .with(alice.as_direct_voter())
            .build();

        let before = db_sync.metadata();
        let file = testing_directory.child("database.json");
        db_sync.persist(file.path()).unwrap();
        let db_sync = InMemoryDbSync::restore(file.path()).unwrap();
        assert_eq!(before, db_sync.metadata());
    }

    #[tokio::test]
    pub async fn dbsync_observer_test() {
        let mut node = InMemoryNode::start(Block0::default());
        let cardano_wallet = CardanoWallet::new(1_000);

        let shared_db_sync = InMemoryDbSync::default().connect_to_node(&mut node);

        node.push_transaction(cardano_wallet.generate_direct_voting_registration(0));

        tokio::time::sleep(Duration::from_secs(
            u64::from(node.settings().unwrap().slot_duration) + 1,
        ))
        .await;

        let db_sync = shared_db_sync.db_sync.read().unwrap();

        assert_eq!(db_sync.blocks.len(), 1);
        assert_eq!(
            db_sync
                .blocks
                .iter()
                .last()
                .unwrap()
                .header()
                .header_body()
                .slot_bignum(),
            BigNum::from(1u32)
        );
        assert_eq!(db_sync.transactions.get(&1).unwrap().len(), 1);
        assert_eq!(db_sync.metadata().get(&1).unwrap().len(), 1);
        assert_eq!(
            db_sync
                .stakes()
                .get(&cardano_wallet.address().to_address().to_hex())
                .unwrap(),
            &BigNum::from(1_000u32)
        );
    }
}
