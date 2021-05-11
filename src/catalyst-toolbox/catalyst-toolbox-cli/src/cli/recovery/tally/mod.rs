pub(crate) mod mockchain;

use chain_core::property::{Deserialize, Fragment};
use chain_impl_mockchain::block::Block;
use jcli_lib::utils::{
    output_file::{Error as OutputFileError, OutputFile},
    output_format::{Error as OutputFormatError, OutputFormat},
};
use jormungandr_lib::interfaces::{
    load_persistent_fragments_logs_from_folder_path, VotePlanStatus,
};

use std::io::{BufReader, Write};
use std::path::PathBuf;

use structopt::StructOpt;

#[allow(clippy::large_enum_variant)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),

    #[error(transparent)]
    DeserializeError(#[from] jormungandr_lib::interfaces::FragmentLogDeserializeError),

    #[error(transparent)]
    LedgerError(#[from] chain_impl_mockchain::ledger::Error),

    #[error("Couldn't initiate a new wallet")]
    WalletError(#[from] jormungandr_testing_utils::wallet::WalletError),

    #[error(transparent)]
    Block0ConfigurationError(#[from] jormungandr_lib::interfaces::Block0ConfigurationError),

    #[error("Block0 do not contain any voteplan")]
    MissingVoteplanError,

    #[error("Could not verify transaction {id} signature with range {range:?}")]
    InvalidTransactionSignature {
        id: String,
        range: std::ops::Range<i32>,
    },

    #[error(transparent)]
    OutputFileError(#[from] OutputFileError),

    #[error(transparent)]
    OutputFormatError(#[from] OutputFormatError),
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Replay {
    #[structopt(long)]
    block0_path: PathBuf,

    #[structopt(long)]
    logs_path: PathBuf,

    #[structopt(flatten)]
    output: OutputFile,

    #[structopt(flatten)]
    output_format: OutputFormat,
}

fn read_block0(path: PathBuf) -> std::io::Result<Block> {
    let reader = std::fs::File::open(path)?;
    Ok(Block::deserialize(BufReader::new(reader)).unwrap())
}

impl Replay {
    pub fn exec(self) -> Result<(), Error> {
        let Replay {
            block0_path,
            logs_path,
            output,
            output_format,
        } = self;
        let block0 = read_block0(block0_path)?;
        let fragments = load_persistent_fragments_logs_from_folder_path(&logs_path)?;

        let (ledger, failed) = mockchain::recover_ledger_from_logs(&block0, fragments)?;
        if !failed.is_empty() {
            println!("{} fragments couldn't be properly processed", failed.len());
            for failed_fragment in failed {
                println!("{}", failed_fragment.id());
            }
        }
        let voteplans = ledger.active_vote_plans();
        let voteplan_status: Vec<VotePlanStatus> =
            voteplans.into_iter().map(VotePlanStatus::from).collect();
        let mut out_writter = output.open()?;
        let content = output_format.format_json(serde_json::to_value(&voteplan_status)?)?;
        out_writter.write_all(content.as_bytes())?;
        Ok(())
    }
}
