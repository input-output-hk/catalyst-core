pub use crate::cardano_node::settings::Settings;
use crate::cardano_node::{Block0, BlockBuilder};
use crate::db_sync::{InMemoryDbSync, SharedInMemoryDbSync};
use crate::wallet::CardanoWallet;
use crate::{InMemoryNode, Ledger};
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::Transaction;
use jormungandr_lib::crypto::account::Identifier;
use std::collections::HashSet;

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
        (db_sync.connect_to_node(&mut node),node, reps)
    }
}

/// Wallet state builder for Network state builder is a trait which creates nice interface for
/// defining role of particular mainnet wallet in voting round. Wallet can be a direct voter/ delegator
/// or representative
pub trait MainnetWalletStateBuilder {
    /// wallet registered as representative. This is simplification and wallet catalyst key is
    /// added to in memory list instead of going through public representative registration process
    fn as_representative(&self) -> MainnetWalletState;

    /// wallet registers as direct voter, meaning it will send self-delegation registration
    fn as_direct_voter(&self) -> MainnetWalletState;
    /// wallet registers as direct voter, meaning it will send self-delegation registration with
    /// given nonce = `slot_no`
    fn as_direct_voter_on_slot_no(&self, slot_no: u64) -> MainnetWalletState;
    /// wallet registers as delegator, meaning it will send delegation registration
    fn as_delegator(&self, delegations: Vec<(&CardanoWallet, u8)>) -> MainnetWalletState;
    /// wallet registers as delegator, meaning it will send delegation registration with
    /// given nonce = `slot_no`
    fn as_delegator_on_slot_no(
        &self,
        delegations: Vec<(&CardanoWallet, u8)>,
        slot_no: u64,
    ) -> MainnetWalletState;
}

impl MainnetWalletStateBuilder for CardanoWallet {
    fn as_representative(&self) -> MainnetWalletState {
        MainnetWalletState {
            rep: Some(self.catalyst_public_key()),
            registration_tx: None,
            stake: self.stake(),
            stake_address: self.reward_address().to_address(),
        }
    }

    fn as_direct_voter(&self) -> MainnetWalletState {
        self.as_direct_voter_on_slot_no(0)
    }

    fn as_direct_voter_on_slot_no(&self, slot_no: u64) -> MainnetWalletState {
        MainnetWalletState {
            rep: None,
            registration_tx: Some(self.generate_direct_voting_registration(slot_no)),
            stake: self.stake(),
            stake_address: self.reward_address().to_address(),
        }
    }

    fn as_delegator(&self, delegations: Vec<(&CardanoWallet, u8)>) -> MainnetWalletState {
        self.as_delegator_on_slot_no(delegations, 0)
    }

    fn as_delegator_on_slot_no(
        &self,
        delegations: Vec<(&CardanoWallet, u8)>,
        slot_no: u64,
    ) -> MainnetWalletState {
        MainnetWalletState {
            rep: None,
            registration_tx: Some(
                self.generate_delegated_voting_registration(
                    delegations
                        .into_iter()
                        .map(|(wallet, weight)| (wallet.catalyst_public_key(), u32::from(weight)))
                        .collect(),
                    slot_no,
                ),
            ),
            stake: self.stake(),
            stake_address: self.reward_address().to_address(),
        }
    }
}

/// Represents wallet candidate for registration. Defines wallet role (delegator/direct-voter/representative)
pub struct MainnetWalletState {
    rep: Option<Identifier>,
    registration_tx: Option<Transaction>,
    stake: u64,
    stake_address: Address,
}

impl MainnetWalletState {
    /// get representative information
    #[must_use]
    pub fn rep(&self) -> &Option<Identifier> {
        &self.rep
    }
    /// get registration metadata
    #[must_use]
    pub fn registration(&self) -> &Option<Transaction> {
        &self.registration_tx
    }
    /// get wallet stake
    #[must_use]
    pub fn stake(&self) -> u64 {
        self.stake
    }
    /// get stake address for wallet
    #[must_use]
    pub fn stake_address(&self) -> &Address {
        &self.stake_address
    }
}
