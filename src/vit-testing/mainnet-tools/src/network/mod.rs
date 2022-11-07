use crate::db_sync::DbSyncInstance;
use crate::wallet::MainnetWallet;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::BlockDate;
use snapshot_lib::registration::VotingRegistration;
use std::collections::HashSet;

pub struct MainnetNetwork<'a> {
    block_date: BlockDate,
    observers: Vec<&'a mut DbSyncInstance>,
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
    pub fn accept(&mut self, registration: VotingRegistration) {
        self.notify_all(self.block_date, registration);
    }

    pub fn sync_with(&mut self, observer: &'a mut DbSyncInstance) {
        self.observers.push(observer);
    }

    fn notify_all(&mut self, block_date: BlockDate, registration: VotingRegistration) {
        for observer in &mut self.observers {
            observer.notify(block_date, registration.clone());
        }
    }
}

#[derive(Default)]
pub struct MainnetNetworkBuilder {
    states: Vec<MainnetWalletState>,
}

impl MainnetNetworkBuilder {
    pub fn with(mut self, state: MainnetWalletState) -> Self {
        self.states.push(state);
        self
    }

    pub fn build(self) -> (DbSyncInstance, HashSet<Identifier>) {
        let mut mainnet_network = MainnetNetwork::default();
        let mut db_sync_instance = DbSyncInstance::default();

        mainnet_network.sync_with(&mut db_sync_instance);

        self.states
            .iter()
            .map(|x| x.registration.as_ref())
            .filter(|x| x.is_some())
            .for_each(|x| mainnet_network.accept(x.unwrap().clone()));

        (
            db_sync_instance,
            self.states
                .iter()
                .map(|x| x.rep.as_ref())
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().clone())
                .collect(),
        )
    }
}

pub trait MainnetWalletStateBuilder {
    fn as_representative(&self) -> MainnetWalletState;
    fn as_direct_voter(&self) -> MainnetWalletState;
    fn as_delegator(&self, delegations: Vec<(&MainnetWallet, u8)>) -> MainnetWalletState;
}

impl MainnetWalletStateBuilder for MainnetWallet {
    fn as_representative(&self) -> MainnetWalletState {
        MainnetWalletState {
            rep: Some(self.catalyst_public_key()),
            registration: None,
        }
    }

    fn as_direct_voter(&self) -> MainnetWalletState {
        MainnetWalletState {
            rep: None,
            registration: Some(self.direct_voting_registration()),
        }
    }

    fn as_delegator(&self, delegations: Vec<(&MainnetWallet, u8)>) -> MainnetWalletState {
        MainnetWalletState {
            rep: None,
            registration: Some(
                self.delegation_voting_registration(
                    delegations
                        .into_iter()
                        .map(|(wallet, weight)| (wallet.catalyst_public_key(), weight as u32))
                        .collect(),
                ),
            ),
        }
    }
}

pub struct MainnetWalletState {
    rep: Option<Identifier>,
    registration: Option<VotingRegistration>,
}

impl MainnetWalletState {
    pub fn rep(&self) -> &Option<Identifier> {
        &self.rep
    }

    pub fn registration(&self) -> &Option<VotingRegistration> {
        &self.registration
    }
}
