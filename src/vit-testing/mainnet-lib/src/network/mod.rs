mod snapshot;
/// wallet state utils
pub mod wallet_state;

use crate::cardano_node::{Block0, BlockBuilder, Settings};
use crate::db_sync::{InMemoryDbSync, SharedInMemoryDbSync};
use crate::{InMemoryNode, Ledger};
use jormungandr_lib::crypto::account::Identifier;
use std::collections::HashSet;

use crate::wallet_state::MainnetWalletState;
pub use snapshot::{Initials, Parameters};

/// Cardano Network state builder, responsible to create a given state of cardano network which will
/// be an input for snapshot
#[derive(Default)]
pub struct MainnetNetworkBuilder {
    states: Vec<MainnetWalletState>,
    settings: Settings,
}

impl MainnetNetworkBuilder {
    #[must_use]
    /// Pushes new wallet to collection
    pub fn with(mut self, state: MainnetWalletState) -> Self {
        self.states.push(state);
        self
    }

    /// Builds block0
    #[must_use]
    pub fn build_block0(&self) -> Block0 {
        let txs: Vec<_> = self
            .states
            .iter()
            .filter_map(|x| x.registration_tx.clone())
            .collect();

        Block0 {
            block: BlockBuilder::next_block(None, &txs),
            settings: self.settings.clone(),
        }
    }

    /// Builds dbsync instance and set or representatives identifiers
    ///
    /// # Panics
    ///
    /// On internal logic issue
    #[must_use]
    pub fn build(self) -> (InMemoryDbSync, Ledger, HashSet<Identifier>) {
        let block0 = self.build_block0();
        let db_sync = InMemoryDbSync::from_block0(&block0);
        let ledger = Ledger::new(block0);

        (
            db_sync,
            ledger,
            self.states
                .iter()
                .map(|x| x.rep.as_ref())
                .filter(Option::is_some)
                .map(|x| x.unwrap().clone())
                .collect(),
        )
    }

    /// Builds dbsync instance and set or representatives identifiers tailored for async usage
    ///
    /// # Panics
    ///
    /// On internal logic issue
    #[must_use]
    pub fn build_shared(self) -> (SharedInMemoryDbSync, InMemoryNode, HashSet<Identifier>) {
        let (db_sync, ledger, reps) = self.build();
        let mut node = InMemoryNode::start_from_ledger(ledger);
        (db_sync.connect_to_node(&mut node), node, reps)
    }
}
