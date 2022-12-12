use cardano_serialization_lib::{AuxiliaryDataSet, Block, Header, HeaderBody, OperationalCert, ProtocolVersion, Transaction, TransactionBodies, TransactionWitnessSets};
use cardano_serialization_lib::crypto::{BlockHash, Ed25519Signature, KESSignature, KESVKey, PrivateKey, Vkey, VRFCert, VRFVKey};
use cardano_serialization_lib::metadata::AuxiliaryData;
use cardano_serialization_lib::utils::BigNum;
use crate::cardano_node::block::{Block0, BlockBuilder};
use crate::Settings;

#[derive(Default)]
pub struct Ledger {
    mempool: Vec<Transaction>,
    blocks: Vec<Block>,
    settings: Settings,
}

impl Ledger {

    pub fn mempool(&self) -> Vec<Transaction> {
        self.mempool.clone()
    }

    pub fn new(block0: Block0) -> Ledger{
        Self {
            mempool: Vec::new(),
            blocks: vec![block0.block],
            settings: block0.settings
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.clone()
    }

    pub fn push_transactions(&mut self, transactions: Vec<Transaction>) {
        self.mempool.extend(transactions);
    }

    pub fn push_transaction(&mut self, transaction: Transaction) {
        self.mempool.push(transaction);
    }

    pub fn mint_block(&mut self) -> Block {
        let last_block = self.blocks.last().expect("internal error no block0 in blockchain");
        let next_block = BlockBuilder::next_block(Some(last_block), self.mempool.clone());
        self.blocks.push(next_block.clone());
        self.mempool.clear();
        next_block
    }
}