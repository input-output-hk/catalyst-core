//!
//! Tool for offline tally
//!

use chain_ser::deser::Deserialize;

use clap::Parser;
use tracing::{info, Level};

use std::{fs::File, io::BufWriter, thread};

use chain_impl_mockchain::block::Block;
use color_eyre::Result;
use jormungandr_lib::interfaces::VotePlanStatus;
use lib::offline::{
    extract_decryption_shares_and_results, extract_fragments_from_storage, json_from_file,
    ledger_after_tally, ledger_before_tally,
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

    // Configure a custom event formatter
    let format = tracing_subscriber::fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(true) // include the thread ID of the current thread
        .with_thread_names(true) // include the name of the current thread
        .compact(); // use the `Compact` formatting style.

    // Create a `fmt` subscriber that uses our custom event format, and set it
    // as the default.
    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(Level::INFO /*DEBUG*/)
        .init();

    info!("Audit Tool.");
    info!("Starting Offline Tally");

    // Load and replay fund fragments from storage
    let storage_path = PathBuf::from(args.fragments);

    // load and read block0
    let block0_path = PathBuf::from(args.block0);
    let block0 = read_block0(block0_path).unwrap();

    // all fragments including tally fragments
    info!("extracting fragments from storage");
    let all_fragments = extract_fragments_from_storage(&storage_path).unwrap();

    // ledger state before tally fragments applied, results are still encrypted. Decrypted results should match results post tally.
    // voting has effectively ended, the results have just not been decrypted yet.
    info!("ledger_before_tally ⏳");
    let fragments_all = all_fragments.clone();
    let block_zero = block0.clone();
    let ledger_before_tally =
        thread::spawn(move || ledger_before_tally(fragments_all, block_zero).unwrap());

    // ledger state after tally fragments applied, results are decrypted i.e encrypted tallies are now plaintext.
    info!("ledger_after_tally ⏳");
    let fragments_all = all_fragments.clone();
    let block_zero = block0;
    let ledger_after_tally =
        thread::spawn(move || ledger_after_tally(fragments_all, block_zero).unwrap());

    let ledger_before_tally = ledger_before_tally.join().unwrap();
    let ledger_after_tally = ledger_after_tally.join().unwrap();
    info!("ledger replays completed ⌛");

    // decrypt_tally_from_shares(pub_keys, encrypted_tally, decrypt_shares) -> tallyResultPlaintext
    // use tally tool to validate decrypted results
    let shares_and_results = extract_decryption_shares_and_results(all_fragments);

    // Compare decrypted tallies with official results if provided
    if let Some(official_results) = args.official_results {
        // official catalyst results in json format
        let official_tallies: Vec<VotePlanStatus> =
            json_from_file(PathBuf::from(official_results))?;

        let matches = official_tallies
            .iter()
            .zip(&ledger_after_tally)
            .filter(|&(a, b)| a == b)
            .count();

        info!("matching re-generated voteplans with given official vote plans.");
        info!("matches: {}/{}", matches, official_tallies.len());
    }

    // write ledger state before and after tally fragments applied to file i.e encrypted and decrypted tallies
    let ledger_before_tally_file =
        PathBuf::from("/tmp/offline").with_extension("ledger_before_tally.json");
    let ledger_after_tally_file =
        PathBuf::from("/tmp/offline").with_extension("ledger_after_tally.json");
    let shares_and_results_file =
        PathBuf::from("/tmp/offline").with_extension("decryption_shares.json");

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(ledger_before_tally_file)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &ledger_before_tally)?;

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(ledger_after_tally_file)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &ledger_after_tally)?;

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(shares_and_results_file)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &shares_and_results)?;

    info!("saved: /tmp/offline.ledger_before_tally.json 🚀");
    info!("saved: /tmp/offline.ledger_after_tally.json 🚀");
    info!("saved: /tmp/offline.decryption_shares.json 🚀");

    Ok(())
}

/// Read block0 from file               
fn read_block0(path: PathBuf) -> Result<Block, Report> {
    let reader = std::fs::File::open(path)?;
    Block::deserialize(&mut Codec::new(reader)).context("block0 loading")
}
