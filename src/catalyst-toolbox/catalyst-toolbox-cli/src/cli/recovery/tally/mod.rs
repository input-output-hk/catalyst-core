pub(crate) mod mockchain;
pub(crate) mod voteplan;

use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DeserializeError(#[from] jormungandr_lib::interfaces::FragmentLogDeserializeError),

    #[error(transparent)]
    LedgerError(#[from] chain_impl_mockchain::ledger::Error),

    #[error("block0 do not contain any voteplan")]
    MissingVoteplanError,
}
