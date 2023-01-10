use crate::cardano_node::block::{Block0, BlockBuilder};
use crate::cardano_node::Settings;
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
    ///
    /// # Errors
    ///
    /// On blockchain inconsistency
    pub fn mint_block(&mut self) -> Result<Block, Error> {
        let last_block = self.blocks.last().ok_or(Error::MissingBlock0)?;
        let next_block = BlockBuilder::next_block(Some(last_block), &self.mempool);
        self.blocks.push(next_block.clone());
        self.mempool.clear();
        Ok(next_block)
    }

    /// Retrieves blockchain in form of vector of blocks
    #[must_use]
    pub fn blockchain(&self) -> &[Block] {
        &self.blocks
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("no block0 in blockchain")]
    MissingBlock0,
}

unsafe impl Send for Error {}
unsafe impl Send for Ledger {}
