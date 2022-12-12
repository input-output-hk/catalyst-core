use crate::cardano_node::block::{Block0, BlockBuilder};
use crate::Settings;
use cardano_serialization_lib::{Block, Transaction};

/// Simulates cardano node behavior. Contains mempool of transactions as well as blockchain.
/// Meant to be used as library in current thread.
#[derive(Default)]
pub struct Ledger {
    mempool: Vec<Transaction>,
    blocks: Vec<Block>,
    settings: Settings,
}

impl Ledger {
    /// Gets current ledger mempool state
    #[must_use]
    pub fn mempool(&self) -> Vec<Transaction> {
        self.mempool.clone()
    }

    /// Creates new ledger based on block0
    #[must_use]
    pub fn new(block0: Block0) -> Ledger {
        Self {
            mempool: Vec::new(),
            blocks: vec![block0.block],
            settings: block0.settings,
        }
    }

    /// Gets node settings
    #[must_use]
    pub fn settings(&self) -> Settings {
        self.settings.clone()
    }

    /// Adds transactions to mempool
    pub fn push_transactions(&mut self, transactions: Vec<Transaction>) {
        self.mempool.extend(transactions);
    }

    /// Add transaction to mempool
    pub fn push_transaction(&mut self, transaction: Transaction) {
        self.mempool.push(transaction);
    }

    /// Mint new block
    pub fn mint_block(&mut self) -> Block {
        let last_block = self
            .blocks
            .last()
            .expect("internal error no block0 in blockchain");
        let next_block = BlockBuilder::next_block(Some(last_block), &self.mempool);
        self.blocks.push(next_block.clone());
        self.mempool.clear();
        next_block
    }
}
