//!
//! Tool for offline Fragment Analysis
//!

use chain_ser::deser::Deserialize;
use clap::Parser;

use std::{fs::File, io::BufWriter};

use chain_impl_mockchain::block::Block;
use color_eyre::Result;
use jormungandr_lib::interfaces::VotePlanStatus;
use lib::{
    offline::{extract_fragments_from_storage, json_from_file},
    recover::recover_ledger_from_logs,
};

use chain_core::packer::Codec;
use color_eyre::{eyre::Context, Report};
use std::{error::Error, path::PathBuf};

///
/// Args defines and declares CLI behaviour within the context of clap
///
#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
pub struct Args {
    /// Obtain fragments by providing path to historical fund data.
    #[clap(short, long)]
    pub fragments: String,
    /// block0 path
    #[clap(short, long)]
    pub block0: String,
    /// cross reference official results
    #[clap(short, long)]
    official_results: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let args = Args::parse();

    // Load and replay fund fragments from storage
    let storage_path = PathBuf::from(args.fragments);

    // load block0
    let block0_path = PathBuf::from(args.block0);

    let block0 = read_block0(block0_path).unwrap();

    let fragments = extract_fragments_from_storage(&storage_path).unwrap();

    let (ledger, failed) = recover_ledger_from_logs(&block0, fragments.into_iter())?;
    if !failed.is_empty() {
        println!("{} fragments couldn't be properly processed", failed.len());
    }

    let voteplans = ledger.active_vote_plans();
    let offline_voteplans: Vec<VotePlanStatus> =
        voteplans.into_iter().map(VotePlanStatus::from).collect();

    // Compare offline tally with official results
    if let Some(official_results) = args.official_results {
        // official catalyst results in json format
        let official_voteplans: Vec<VotePlanStatus> =
            json_from_file(PathBuf::from(official_results))?;

        let matches = official_voteplans
            .iter()
            .zip(&offline_voteplans)
            .filter(|&(a, b)| a == b)
            .count();

        println!("matches: {}/{}", matches, official_voteplans.len());
    }

    let offline = PathBuf::from("/tmp/offline").with_extension("offline_tally.json");

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(offline)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &offline_voteplans)?;

    Ok(())
}

fn read_block0(path: PathBuf) -> Result<Block, Report> {
    let reader = std::fs::File::open(path)?;
    Block::deserialize(&mut Codec::new(reader)).context("block0 loading")
}
