//!
//! Find my vote
//!

use clap::Parser;
use lib::find::{all_voters, convert_key_formats, find_vote, read_lines};
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
    pub fragments: Option<String>,
    /// voting key
    #[clap(short, long, requires = "fragments")]
    voting_key: Option<String>,
    /// aggregate voting keys
    #[clap(short, long, requires = "fragments")]
    aggregate: Option<bool>,
    ///convert key formats
    #[clap(short, long)]
    key_to_convert: Option<String>,
    /// check batch of keys and write history to file
    #[clap(short, long)]
    key_file: Option<String>,
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

    if let Some(voting_key) = args.voting_key {
        // Load and replay fund fragments from storage
        let storage_path = PathBuf::from(
            args.fragments
                .clone()
                .expect("enforced by clap: infallible"),
        );

        // all fragments including tally fragments
        info!("finding vote history of voter {:?}", voting_key);

        let matched_votes = find_vote(&storage_path, voting_key.clone())?;

        // record of casters votes
        let matched_votes_path = PathBuf::from("/tmp/offline")
            .with_extension(format!("voting_history_of_{}.json", voting_key));

        let file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(matched_votes_path.clone())?;
        let writer = BufWriter::new(file);

        info!(
            "writing voting history of voter {:?} to {:?}",
            voting_key, matched_votes_path
        );

        serde_json::to_writer_pretty(writer, &matched_votes)?;
    }

    if let Some(_aggregate) = args.aggregate {
        // Load and replay fund fragments from storage
        let storage_path = PathBuf::from(args.fragments.expect("enforced by clap: infallible"));

        info!("collecting all voting keys in ca and 0x format");

        let (unique_voters_ca, unique_voters_0x) = all_voters(&storage_path)?;

        let voters_file_0x =
            PathBuf::from("/tmp/inspect").with_extension("validated_voters_0x.json");
        let voters_file_ca =
            PathBuf::from("/tmp/inspect").with_extension("validated_voters_ca.json");

        let file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(voters_file_ca)
            .unwrap();
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &unique_voters_ca)?;

        let file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(voters_file_0x)
            .unwrap();
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &unique_voters_0x)?;

        info!("keys written to /tmp/inspect/validated_voters_*.json");
    }

    if let Some(keyfile) = args.key_file {
        let keys = read_lines(&keyfile);

        batch_key_check(jormungandr_database, key_file)
    }

    if let Some(voting_key) = args.key_to_convert {
        let converted_key = convert_key_formats(voting_key)?;
        info!("Converted key: {}", converted_key);
    }

    Ok(())
}
