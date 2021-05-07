pub(crate) mod mockchain;
pub(crate) mod voteplan;

use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DeserializeError(#[from] jormungandr_lib::interfaces::FragmentLogDeserializeError),

    #[error(transparent)]
    LedgerError(#[from] chain_impl_mockchain::ledger::Error),

    #[error("Couldn't initiate a new wallet")]
    WalletError(#[from] jormungandr_testing_utils::wallet::WalletError),

    #[error(transparent)]
    Block0ConfigurationError(#[from] jormungandr_lib::interfaces::Block0ConfigurationError),

    #[error("block0 do not contain any voteplan")]
    MissingVoteplanError,
}
