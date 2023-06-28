//!
//! Tool for offline Fragment Analysis
//!

use clap::Parser;

use color_eyre::Result;
use jormungandr_lib::interfaces::VotePlanStatus;
use lib::offline::{extract_tally_fragments, json_from_file};

use std::{error::Error, path::PathBuf};

///
/// Args defines and declares CLI behaviour within the context of clap
///
#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
pub struct Args {
    /// Obtain tally fragments by providing path to historical fund data.
    #[clap(short, long)]
    pub tally_fragments: String,
    /// Official catalyst results in json format.
    #[clap(short, long)]
    pub active_vote_plans: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let args = Args::parse();

    // Load and replay fund fragments from storage
    let storage_path = PathBuf::from(args.tally_fragments);

    let active_plans_official_path = PathBuf::from(args.active_vote_plans);

    // extracted tally fragments (results) from replayed fund
    let tallies = extract_tally_fragments(&storage_path)?;

    // official catalyst results in json format
    let voteplans: Vec<VotePlanStatus> = json_from_file(active_plans_official_path)?;

    // cross reference official results with tally fragments

    println!("votes {:?}", voteplans);

    Ok(())
}
