use chain_addr::{AddressReadable, Discrimination};
use chain_impl_mockchain::account::SpendingCounterIncreasing;
use chain_impl_mockchain::{block::BlockDate, fragment::FragmentId};
use hdkeygen::account::AccountId;
use jormungandr_lib::interfaces::AccountIdentifier;
use std::str::FromStr;
use thiserror::Error;
use wallet::Settings;
use wallet_core::Proposal;
use wallet_core::Wallet as Inner;
use wallet_core::{Choice, Value};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot recover from mnemonics: {0}")]
    CannotRecover(String),
    #[error("cannot retrieve funds: {0}")]
    CannotRetrieveFunds(String),
    #[error("backend error")]
    BackendError(#[from] valgrind::Error),
    #[error("cannot send vote")]
    CannotSendVote(String),
}

pub struct Wallet {
    inner: Inner,
    pending_txs: Vec<FragmentId>,
}

impl Wallet {
    pub fn recover(secret_key: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::recover_free_keys(secret_key)
                .map_err(|e| Error::CannotRecover(e.to_string()))?,
            pending_txs: Vec::new(),
        })
    }

    pub fn recover_from_utxo(secret_key: &[u8; 64]) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::recover_free_keys(secret_key)
                .map_err(|e| Error::CannotRecover(e.to_string()))?,
            pending_txs: Vec::new(),
        })
    }

    pub fn account(&self, discrimination: chain_addr::Discrimination) -> chain_addr::Address {
        self.inner.account(discrimination)
    }

    pub fn id(&self) -> AccountId {
        self.inner.id()
    }

    pub fn confirm_all_transactions(&mut self) {
        for id in self.pending_transactions() {
            self.confirm_transaction(id)
        }
    }

    pub fn confirm_transaction(&mut self, id: FragmentId) {
        self.inner.confirm_transaction(id);
        self.remove_pending_transaction(id);
    }

    pub fn pending_transactions(&self) -> Vec<FragmentId> {
        self.pending_txs.clone()
    }

    pub fn remove_pending_transaction(&mut self, id: FragmentId) {
        if let Some(index) = self.pending_txs.iter().position(|x| *x == id) {
            self.pending_txs.remove(index);
        }
    }

    pub fn total_value(&self) -> Value {
        self.inner.total_value()
    }

    pub fn set_state(&mut self, value: Value, counters: [u32; SpendingCounterIncreasing::LANES]) {
        //TODO map error instead of unwrapping
        self.inner.set_state(value, counters).unwrap();
    }

    pub fn spending_counter(&self) -> [u32; SpendingCounterIncreasing::LANES] {
        self.inner.spending_counter()
    }

    pub fn vote(
        &mut self,
        settings: Settings,
        proposal: &Proposal,
        choice: Choice,
        valid_until: &BlockDate,
    ) -> Result<Box<[u8]>, Error> {
        self.inner
            .vote(settings, proposal, choice, valid_until, 0u8)
            .map_err(|e| Error::CannotSendVote(e.to_string()))
    }

    pub fn identifier(&self, discrimination: Discrimination) -> AccountIdentifier {
        let address_readable = match discrimination {
            Discrimination::Test => {
                AddressReadable::from_address("ta", &self.account(discrimination)).to_string()
            }
            Discrimination::Production => {
                AddressReadable::from_address("ca", &self.account(discrimination)).to_string()
            }
        };
        AccountIdentifier::from_str(&address_readable).unwrap()
    }
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier(Discrimination::Production))
    }
}
