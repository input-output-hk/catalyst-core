//!
//! Tool for offline Fragment Analysis
//!

use chain_ser::deser::Deserialize;
use clap::Parser;

use std::{fs::File, io::BufWriter};

use chain_impl_mockchain::block::Block;
use color_eyre::Result;
use jormungandr_lib::interfaces::VotePlanStatus;
use lib::offline::{extract_fragments_from_storage, get_decrypted_tallies, json_from_file};

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

    // load and read block0
    let block0_path = PathBuf::from(args.block0);
    let block0 = read_block0(block0_path).unwrap();

    // all fragments including tally fragments
    let all_fragments = extract_fragments_from_storage(&storage_path).unwrap();

    //let encrypted_tallies = get_encrypted_tallies(all_fragments.clone(), block0.clone())?;

    let decrypted_tallies = get_decrypted_tallies(all_fragments.clone(), block0.clone())?;

    // Compare decrypted tallies with official results if provided
    if let Some(official_results) = args.official_results {
        // official catalyst results in json format
        let official_tallies: Vec<VotePlanStatus> =
            json_from_file(PathBuf::from(official_results))?;

        let matches = official_tallies
            .iter()
            .zip(&decrypted_tallies)
            .filter(|&(a, b)| a == b)
            .count();

        println!("matches: {}/{}", matches, official_tallies.len());
    }

    // write decrypted tallies to file
    let offline = PathBuf::from("/tmp/offline").with_extension("decrypted_tally.json");

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(offline)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &decrypted_tallies)?;

    Ok(())
}

/// Read block0 from file               
fn read_block0(path: PathBuf) -> Result<Block, Report> {
    let reader = std::fs::File::open(path)?;
    Block::deserialize(&mut Codec::new(reader)).context("block0 loading")
}
