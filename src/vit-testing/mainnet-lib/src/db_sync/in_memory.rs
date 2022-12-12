use cardano_serialization_lib::metadata::GeneralTransactionMetadata;
use jormungandr_lib::interfaces::BlockDate;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use cardano_serialization_lib::{Block, Transaction, TransactionWitnessSet};
use cardano_serialization_lib::utils::BigNum;
use futures::executor::block_on;
use serde::{Deserialize,Serialize};
use pharos::{Events,Channel,Observable};
use crate::InMemoryNode;

const CARDANO_MAINNET_SLOTS_PER_EPOCH: u64 = 43200;

pub type BlockNo = u32;
pub type Address = String;


/// Mock of real cardano db sync. At this moment we only stores transactions metadata
/// as the only purpose of existance for this struct is to provide catalyst voting registrations
/// Struct can be persisted and restored from json file using `serde_json`.
#[derive(Serialize, Deserialize, Default)]
pub struct InMemoryDbSync {
    pub(crate) transactions: HashMap<BlockNo,Vec<Transaction>>,
    pub(crate) blocks: Vec<Block>,
    stakes: HashMap<Address, BigNum>,
    settings: Settings,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub(crate) node_observer: Option<Events<Block>>
}

impl Debug for InMemoryDbSync {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.settings)
    }
}

impl InMemoryDbSync {

    /// Connects to Cardano mock node using simple observer/observable mechanism
    pub fn connect_to_node(&mut self, node: &mut InMemoryNode) {
        let observer = block_on(async {
            node.observe( Channel::Bounded( 3 ).into() ).await.expect("observer")
        });
        self.node_observer = Some(observer);
    }

    /// Retrieves db sync content as string
    ///
    /// # Errors
    ///
    /// On deserialization issues
    pub fn try_as_string(&self) -> Result<String,serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    /// Accept new block
    pub fn on_block_propagation(
        &mut self,
        block: &Block,
    ) {
        self.blocks.push(block.clone());

        let mut transactions = vec![];
        let bodies = block.transaction_bodies();
        for i  in 0..bodies.len() {
            let outputs = bodies.get(i).outputs();

            for i in 0..outputs.len() {
                let output = outputs.get(i);
                let stake = output.amount().coin();
                self.stakes.entry(output.address().to_hex()).and_modify(|x| *x = x.checked_add(&stake).unwrap() ).or_insert(stake);
            }

            transactions.push(Transaction::new(&bodies.get(i), &TransactionWitnessSet::new(), block.auxiliary_data_set().get(i as u32)));
        }

        self.transactions.insert(block.header().header_body().block_number(), transactions);
    }

    pub fn transaction_by_hash(&self, hash: &str) -> Vec<(&Block,&Transaction)> {
        self.transactions.iter().filter_map(|(block, txs)| {
            if let Some(tx) = txs.iter().find(|tx| tx.to_hex() == hash) {
                let block = self.blocks.iter().find(|x| x.header().header_body().block_number() == *block).expect("data inconsitency. cannot find block with block no from tx");

                return Some((block,tx));
            }
            None
        }).collect()
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

    pub fn metadata(&self) -> HashMap<BlockNo,Vec<GeneralTransactionMetadata>> {
        self.transactions.iter().map(|(block,tx)| {
            let metadata = tx.iter().filter_map(|x| {
                if let Some(auxiliary_data) = x.auxiliary_data() {
                    if let Some(metadata) = auxiliary_data.metadata() {
                        return Some(metadata.clone());
                    }
                }
                None
            }).collect();
            (*block, metadata)
        }).collect()
    }

    /// Gets all transactions metadata with `slot_no` upper and lower bounds
    pub fn query_voting_transactions_with_bounds(
        &self,
        lower: Option<u64>,
        upper: Option<u64>,
    ) -> HashMap<BlockNo, Vec<GeneralTransactionMetadata>> {

       self.metadata().into_iter()
            .filter(|(block_no, _)| {
                if let Some(result) = lower.map(|x| (x as u32) > *block_no) {
                    if result {
                        return false;
                    }
                }
                if let Some(result) = upper.map(|x| (x as u32) < *block_no) {
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
    pub fn stakes(&self) -> &HashMap<String, BigNum> {
        &self.stakes
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
    fn to_absolute_slot_no(self) -> u64;
}

impl BlockDateFromCardanoAbsoluteSlotNo for BlockDate {
    fn from_absolute_slot_no(absolute_slot_no: u64) -> Self {
        let epoch = absolute_slot_no / CARDANO_MAINNET_SLOTS_PER_EPOCH;
        let slot = absolute_slot_no - epoch * CARDANO_MAINNET_SLOTS_PER_EPOCH;
        BlockDate::new(u32::try_from(epoch).unwrap(), u32::try_from(slot).unwrap())
    }

    fn to_absolute_slot_no(self) -> u64 {
        (self.epoch() * (CARDANO_MAINNET_SLOTS_PER_EPOCH as u32) + self.slot()) as u64
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::fixture::PathChild;
    use crate::{JsonBasedDbSync, MainnetNetworkBuilder, CardanoWallet, MainnetWalletStateBuilder};
    use assert_fs::TempDir;

    #[test]
    fn restore_persist_bijection_direct() {
        let testing_directory = TempDir::new().unwrap();

        let alice = CardanoWallet::new(1_000);

        let (db_sync, _node,  _reps) = MainnetNetworkBuilder::default()
            .with(alice.as_direct_voter())
            .as_json(testing_directory.child("database.json").path());

        let before = db_sync.tx_metadata().clone();
        db_sync.persist().unwrap();
        let db_sync = JsonBasedDbSync::restore(&db_sync.db_file).unwrap();
        assert_eq!(before, db_sync.tx_metadata());
    }
}
