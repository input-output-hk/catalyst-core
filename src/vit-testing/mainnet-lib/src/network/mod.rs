use crate::db_sync::InMemoryDbSync;
use crate::wallet::MainnetWallet;
use assert_fs::TempDir;
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::metadata::GeneralTransactionMetadata;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::BlockDate;
use std::collections::HashSet;

/// Mainnet network mock. It holds current block date as indicator on which epoch and slot mainnet
/// network currently is. Also struct can have multiple db sync which once register can be notified
/// on incoming transactions.
pub struct MainnetNetwork<'a> {
    block_date: BlockDate,
    observers: Vec<&'a mut InMemoryDbSync>,
}

impl Default for MainnetNetwork<'_> {
    fn default() -> Self {
        Self {
            block_date: BlockDate::new(0, 0),
            observers: vec![],
        }
    }
}

impl<'a> MainnetNetwork<'a> {
    /// accepts registration tx in form of raw metadata and notifies all db sync instances
    pub fn accept_registration_tx(&mut self, registration: &GeneralTransactionMetadata) {
        for observer in &mut self.observers {
            observer.push_transaction(self.block_date, registration.clone());
        }
    }

    /// register db sync instance as observer, which will be notified on each incoming transaction
    pub fn sync_with(&mut self, observer: &'a mut InMemoryDbSync) {
        self.observers.push(observer);
    }

    /// accept address and it's ada value. This is simplification for calculating wallet stake, since
    /// out point of focus are only registration transactions
    pub fn accept_address(&mut self, address: &Address, stake: u64) {
        for observer in &mut self.observers {
            observer.push_address(address, stake);
        }
    }
}

/// Cardano Network state builder, responsible to create a given state of cardano network which will
/// be an input for snapshot
#[derive(Default)]
pub struct MainnetNetworkBuilder {
    states: Vec<MainnetWalletState>,
}

impl MainnetNetworkBuilder {
    #[must_use]
    /// Pushes new wallet to collection
    pub fn with(mut self, state: MainnetWalletState) -> Self {
        self.states.push(state);
        self
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    /// Builds dbsync instance and set or representatives identifiers
    pub fn build(self, temp_dir: &TempDir) -> (InMemoryDbSync, HashSet<Identifier>) {
        let mut mainnet_network = MainnetNetwork::default();
        let mut db_sync_instance = InMemoryDbSync::new(temp_dir);

        mainnet_network.sync_with(&mut db_sync_instance);

        self.states.iter().for_each(|x| {
            if let Some(tx) = &x.registration {
                mainnet_network.accept_registration_tx(tx);
            }
            mainnet_network.accept_address(&x.stake_address, x.stake);
        });

        (
            db_sync_instance,
            self.states
                .iter()
                .map(|x| x.rep.as_ref())
                .filter(Option::is_some)
                .map(|x| x.unwrap().clone())
                .collect(),
        )
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
    fn as_delegator(&self, delegations: Vec<(&MainnetWallet, u8)>) -> MainnetWalletState;
    /// wallet registers as delegator, meaning it will send delegation registration with
    /// given nonce = `slot_no`
    fn as_delegator_on_slot_no(
        &self,
        delegations: Vec<(&MainnetWallet, u8)>,
        slot_no: u64,
    ) -> MainnetWalletState;
}

impl MainnetWalletStateBuilder for MainnetWallet {
    fn as_representative(&self) -> MainnetWalletState {
        MainnetWalletState {
            rep: Some(self.catalyst_public_key()),
            registration: None,
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
            registration: Some(self.generate_direct_voting_registration(slot_no)),
            stake: self.stake(),
            stake_address: self.reward_address().to_address(),
        }
    }

    fn as_delegator(&self, delegations: Vec<(&MainnetWallet, u8)>) -> MainnetWalletState {
        self.as_delegator_on_slot_no(delegations, 0)
    }

    fn as_delegator_on_slot_no(
        &self,
        delegations: Vec<(&MainnetWallet, u8)>,
        slot_no: u64,
    ) -> MainnetWalletState {
        MainnetWalletState {
            rep: None,
            registration: Some(
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
    registration: Option<GeneralTransactionMetadata>,
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
    pub fn registration(&self) -> &Option<GeneralTransactionMetadata> {
        &self.registration
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
