use catalyst_toolbox_lib::recovery::tally::recover_ledger_from_logs;
use chain_core::property::{Deserialize, Fragment};
use chain_impl_mockchain::block::Block;
use jcli_lib::utils::{
    output_file::{Error as OutputFileError, OutputFile},
    output_format::{Error as OutputFormatError, OutputFormat},
};
use jormungandr_lib::interfaces::{
    load_persistent_fragments_logs_from_folder_path, VotePlanStatus,
};

use log::warn;
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
    RecoveryError(#[from] catalyst_toolbox_lib::recovery::tally::Error),

    #[error(transparent)]
    OutputFileError(#[from] OutputFileError),

    #[error(transparent)]
    OutputFormatError(#[from] OutputFormatError),
}

/// Recover the tally from fragment log files and the initial preloaded block0 binary file.
#[derive(StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Replay {
    /// Path to the block0 binary file
    #[structopt(long)]
    block0_path: PathBuf,

    /// Path to the folder containing the log files used for the tally reconstruction
    #[structopt(long)]
    logs_path: PathBuf,

    #[structopt(flatten)]
    output: OutputFile,

    #[structopt(flatten)]
    output_format: OutputFormat,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
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
            verbose,
        } = self;
        stderrlog::new().verbosity(verbose).init().unwrap();
        let block0 = read_block0(block0_path)?;

        let fragments = load_persistent_fragments_logs_from_folder_path(&logs_path)?;

        let (ledger, failed) = recover_ledger_from_logs(&block0, fragments)?;
        if !failed.is_empty() {
            warn!("{} fragments couldn't be properly processed", failed.len());
            for failed_fragment in failed {
                warn!("{}", failed_fragment.id());
            }
        }
        let voteplans = ledger.active_vote_plans();
        let voteplan_status: Vec<VotePlanStatus> =
            voteplans.into_iter().map(VotePlanStatus::from).collect();
        let mut out_writer = output.open()?;
        let content = output_format.format_json(serde_json::to_value(&voteplan_status)?)?;
        out_writer.write_all(content.as_bytes())?;
        Ok(())
    }
}
