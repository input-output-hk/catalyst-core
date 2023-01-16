use catalyst_toolbox::recovery::Replay;
use chain_core::{packer::Codec, property::Deserialize};
use chain_impl_mockchain::block::Block;
use color_eyre::{
    eyre::{bail, Context},
    Report,
};
use jcli_lib::utils::{output_file::OutputFile, output_format::OutputFormat};

use std::path::PathBuf;

use clap::Parser;
use reqwest::Url;

use super::set_verbosity;

/// Recover the tally from fragment log files and the initial preloaded block0 binary file.
#[derive(Parser)]
#[clap(rename_all = "kebab")]
pub struct ReplayCli {
    /// Path to the block0 binary file
    #[clap(long, conflicts_with = "block0-url")]
    block0_path: Option<PathBuf>,

    /// Url to a block0 endpoint
    #[clap(long)]
    block0_url: Option<Url>,

    /// Path to the folder containing the log files used for the tally reconstruction
    #[clap(long)]
    logs_path: PathBuf,

    #[clap(flatten)]
    output: OutputFile,

    #[clap(flatten)]
    output_format: OutputFormat,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: usize,
}

fn read_block0(path: PathBuf) -> Result<Block, Report> {
    let reader = std::fs::File::open(path)?;
    Block::deserialize(&mut Codec::new(reader)).context("block0 loading")
}

fn load_block0_from_url(url: Url) -> Result<Block, Report> {
    let block0_body = reqwest::blocking::get(url)?.bytes()?;
    Block::deserialize(&mut Codec::new(&block0_body[..])).context("block0 loading")
}

impl ReplayCli {
    pub fn exec(self) -> Result<(), Report> {
        let Self {
            block0_path,
            block0_url,
            logs_path,
            output,
            output_format,
            verbose,
        } = self;

        set_verbosity(verbose);

        let block0 = if let Some(path) = block0_path {
            read_block0(path)?
        } else if let Some(url) = block0_url {
            load_block0_from_url(url)?
        } else {
            bail!("block0 unavailable");
        };

        let replay = Replay::new(block0, logs_path, output, output_format);
        replay.exec().map_err(Into::into)
    }
}
