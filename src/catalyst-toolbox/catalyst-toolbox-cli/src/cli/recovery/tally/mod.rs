pub(crate) mod mockchain;

use chain_core::property::Deserialize as _;
use chain_impl_mockchain::block::Block;
use jormungandr_lib::interfaces::load_persistent_fragments_logs_from_folder_path;

use std::io::BufReader;
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

fn read_block0(path: PathBuf) -> std::io::Result<Block> {
    let reader = std::fs::File::open(path)?;
    Ok(Block::deserialize(BufReader::new(reader)).unwrap())
}

impl Replay {
    pub fn exec(self) -> Result<(), Error> {
        // let Replay {
        //     block0_path,
        //     logs_path,
        // } = self;
        // let block0 = read_block0(block0_path)?;
        // let fragments = load_persistent_fragments_logs_from_folder_path(&logs_path)?;
        //
        // let (ledger, failed) = mockchain::recover_ledger_from_logs(&block0, fragments)?;

        Ok(())
    }
}
