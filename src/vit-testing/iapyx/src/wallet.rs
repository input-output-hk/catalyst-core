use chain_addr::{AddressReadable, Discrimination};
use chain_impl_mockchain::account::SpendingCounterIncreasing;
use chain_impl_mockchain::{block::BlockDate, fragment::FragmentId};
use hdkeygen::account::AccountId;
use jormungandr_lib::interfaces::{AccountIdentifier, ParseAccountIdentifierError};
use std::str::FromStr;
use thiserror::Error;
use thor::DiscriminationExtension;
use wallet::Settings;
use wallet_core::Proposal;
use wallet_core::Wallet as Inner;
use wallet_core::{Choice, Value};

/// Wallet error
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    /// Recovery failed
    #[error("cannot recover from mnemonics: {0}")]
    CannotRecover(String),
    /// Funds retrieval failure
    #[error("cannot retrieve funds: {0}")]
    CannotRetrieveFunds(String),
    /// Backend issues
    #[error("backend error")]
    BackendError(#[from] valgrind::Error),
    /// Error with sending votes
    #[error("cannot send vote")]
    CannotSendVote(String),
}

/// Wallet object, which wraps struct from chain-wallet-libs project in order to preserve production
/// behavior for wallet. One addition to that struct is a cache of recently send transaction which
/// needs to be "confirmed" to remove any data which enable track them in blockchain
pub struct Wallet {
    inner: Inner,
    pending_txs: Vec<FragmentId>,
}

impl Wallet {
    /// Recover wallet from secret key bytes
    ///
    /// # Errors
    ///
    ///  On recovery process failure
    pub fn recover(secret_key: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::recover_free_keys(secret_key)
                .map_err(|e| Error::CannotRecover(e.to_string()))?,
            pending_txs: Vec::new(),
        })
    }

    /// Recover from utxo secret key    ///
    /// # Errors
    ///
    ///  On recovery process failure
    pub fn recover_from_utxo(secret_key: &[u8; 64]) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::recover_free_keys(secret_key)
                .map_err(|e| Error::CannotRecover(e.to_string()))?,
            pending_txs: Vec::new(),
        })
    }

    /// Get account object
    pub fn account(&self, discrimination: Discrimination) -> chain_addr::Address {
        self.inner.account(discrimination)
    }

    /// Get account id
    pub fn id(&self) -> AccountId {
        self.inner.id()
    }

    /// Confirms all transactions means to remove them from pending transaction collection and as a result
    /// remove trace needed to track their status in node
    pub fn confirm_all_transactions(&mut self) {
        for id in self.pending_transactions() {
            self.confirm_transaction(id)
        }
    }

    /// Confirms transaction by it id. This means to remove it from pending transaction collection and as a result
    /// remove trace needed to track their status in node
    pub fn confirm_transaction(&mut self, id: FragmentId) {
        self.inner.confirm_transaction(id);
        self.remove_pending_transaction(id);
    }

    /// Unconfirmed collection of transactions (which statuses we still want to track)
    pub fn pending_transactions(&self) -> Vec<FragmentId> {
        self.pending_txs.clone()
    }

    /// remove specific transaction from assumed pending
    pub fn remove_pending_transaction(&mut self, id: FragmentId) {
        if let Some(index) = self.pending_txs.iter().position(|x| *x == id) {
            self.pending_txs.remove(index);
        }
    }

    /// total ada wallet holds
    pub fn total_value(&self) -> Value {
        self.inner.total_value()
    }

    /// update value and spending-counters
    ///
    /// # Errors
    ///
    /// On invalid spending counter errors
    pub fn set_state(
        &mut self,
        value: Value,
        counters: [u32; SpendingCounterIncreasing::LANES],
    ) -> Result<(), wallet_core::Error> {
        self.inner.set_state(value, counters)
    }

    /// Gets spending counters
    pub fn spending_counter(&self) -> [u32; SpendingCounterIncreasing::LANES] {
        self.inner.spending_counter()
    }

    /// Send specialized transaction (with vote certificates) based on parameters
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
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

    /// Get account identifier
    ///
    /// # Errors
    ///
    /// On parse account error
    pub fn identifier(
        &self,
        discrimination: Discrimination,
    ) -> Result<AccountIdentifier, ParseAccountIdentifierError> {
        let address_readable = AddressReadable::from_address(
            &discrimination.into_prefix(),
            &self.account(discrimination),
        )
        .to_string();
        AccountIdentifier::from_str(&address_readable)
    }
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.identifier(Discrimination::Production).unwrap()
        )
    }
}
