use catalyst_toolbox::recovery::{Replay, ReplayError};
use chain_core::{packer::Codec, property::Deserialize};
use chain_impl_mockchain::block::Block;
use jcli_lib::utils::{
    output_file::{Error as OutputFileError, OutputFile},
    output_format::{Error as OutputFormatError, OutputFormat},
};

use std::path::PathBuf;

use reqwest::Url;
use structopt::StructOpt;

#[allow(clippy::large_enum_variant)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Replay(#[from] ReplayError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    OutputFile(#[from] OutputFileError),

    #[error(transparent)]
    OutputFormat(#[from] OutputFormatError),

    #[error("Block0 should be provided either from a path (block0-path) or an url (block0-url)")]
    Block0Unavailable,

    #[error("Could not load block0")]
    Block0Loading(#[source] std::io::Error),
}

/// Recover the tally from fragment log files and the initial preloaded block0 binary file.
#[derive(StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct ReplayCli {
    /// Path to the block0 binary file
    #[structopt(long, conflicts_with = "block0-url")]
    block0_path: Option<PathBuf>,

    /// Url to a block0 endpoint
    #[structopt(long)]
    block0_url: Option<Url>,

    /// Path to the folder containing the log files used for the tally reconstruction
    #[structopt(long)]
    logs_path: PathBuf,

    #[structopt(flatten)]
    output: OutputFile,

    #[structopt(flatten)]
    output_format: OutputFormat,

    /// Verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: log::LevelFilter,
}

fn read_block0(path: PathBuf) -> Result<Block, Error> {
    let reader = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(reader);
    Block::deserialize(&mut Codec::new(reader)).map_err(Error::Block0Loading)
}

fn load_block0_from_url(url: Url) -> Result<Block, Error> {
    let block0_body = reqwest::blocking::get(url)?.bytes()?;
    Block::deserialize(&mut Codec::new(&block0_body[..])).map_err(Error::Block0Loading)
}

impl ReplayCli {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            block0_path,
            block0_url,
            logs_path,
            output,
            output_format,
            verbose,
        } = self;
        env_logger::Builder::new().filter_level(verbose).init();

        let block0 = if let Some(path) = block0_path {
            read_block0(path)?
        } else if let Some(url) = block0_url {
            load_block0_from_url(url)?
        } else {
            return Err(Error::Block0Unavailable);
        };

        let replay = Replay::new(block0, logs_path, output, output_format);
        replay.exec().map_err(Into::into)
    }
}
