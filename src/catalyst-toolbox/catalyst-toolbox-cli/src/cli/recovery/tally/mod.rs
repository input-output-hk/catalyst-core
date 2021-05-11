pub(crate) mod mockchain;

use std::path::PathBuf;
use structopt::StructOpt;

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

    #[error("Could not verify transaction {id} signature with range {range:?}")]
    InvalidTransactionSignature {
        id: String,
        range: std::ops::Range<i32>,
    },
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Replay {
    block0_path: PathBuf,
    logs_path: PathBuf,
}

impl Replay {
    pub fn exec(self) -> Result<(), Error> {
        Ok(())
    }
}
