use crate::recovery::tally::recover_ledger_from_logs;
use chain_core::property::Fragment;
use chain_impl_mockchain::block::Block;
pub use jcli_lib::utils::{
    output_file::{Error as OutputFileError, OutputFile},
    output_format::{Error as OutputFormatError, OutputFormat},
};
use jormungandr_lib::interfaces::{
    load_persistent_fragments_logs_from_folder_path, VotePlanStatus,
};
use log::warn;
use std::io::Write;
use std::path::PathBuf;

/// Recover the tally from fragment log files and the initial preloaded block0 binary file.
pub struct Replay {
    block0: Block,
    /// Path to the folder containing the log files used for the tally reconstruction
    logs_path: PathBuf,
    output: OutputFile,
    output_format: OutputFormat,
}

impl Replay {
    pub fn new(
        block0: Block,
        logs_path: PathBuf,
        output: OutputFile,
        output_format: OutputFormat,
    ) -> Self {
        Self {
            block0,
            logs_path,
            output,
            output_format,
        }
    }

    pub fn exec(self) -> Result<(), Error> {
        let fragments = load_persistent_fragments_logs_from_folder_path(&self.logs_path)
            .map_err(Error::PersistenLogsLoading)?;

        let (ledger, failed) = recover_ledger_from_logs(&self.block0, fragments)?;
        if !failed.is_empty() {
            warn!("{} fragments couldn't be properly processed", failed.len());
            for failed_fragment in failed {
                warn!("{}", failed_fragment.id());
            }
        }
        let voteplans = ledger.active_vote_plans();
        let voteplan_status: Vec<VotePlanStatus> =
            voteplans.into_iter().map(VotePlanStatus::from).collect();
        let mut out_writer = self.output.open()?;
        let content = self
            .output_format
            .format_json(serde_json::to_value(&voteplan_status)?)?;
        out_writer.write_all(content.as_bytes())?;
        Ok(())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Recovery(#[from] crate::recovery::tally::Error),

    #[error(transparent)]
    OutputFile(#[from] OutputFileError),

    #[error(transparent)]
    OutputFormat(#[from] OutputFormatError),

    #[error("Could not load persistent logs from path")]
    PersistenLogsLoading(#[source] std::io::Error),
}
