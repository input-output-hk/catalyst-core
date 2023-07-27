//!
//! Find my vote
//!

use clap::Parser;
use lib::find::find_vote;
use tracing::{info, Level};

use color_eyre::Result;

use std::{fs::File, io::BufWriter};

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
    /// voting key
    #[clap(short, long)]
    voting_key: String,
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
    info!("Find my vote");

    // Load and replay fund fragments from storage
    let storage_path = PathBuf::from(args.fragments);

    // all fragments including tally fragments
    info!("finding vote history of voter {:?}", args.voting_key);

    let matched_votes = find_vote(&storage_path, args.voting_key.clone())?;

    // record of casters votes
    let matched_votes_path = PathBuf::from("/tmp/offline")
        .with_extension(format!("voting_history_of_{}.json", args.voting_key));

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(matched_votes_path.clone())?;
    let writer = BufWriter::new(file);

    info!(
        "writing voting history of voter {:?} to {:?}",
        args.voting_key, matched_votes_path
    );

    serde_json::to_writer_pretty(writer, &matched_votes)?;

    Ok(())
}
