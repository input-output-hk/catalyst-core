//!
//! Tool for Fragment Analysis
//!

use clap::Parser;

use color_eyre::Result;
use lib::fragment_analysis::extract_tally_fragments;

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
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let args = Args::parse();

    //
    // Load and replay fund fragments from storage
    //
    let storage_path = args.tally_fragments;

    let path = PathBuf::from(storage_path);

    let tallies = extract_tally_fragments(&path).unwrap();

    // match against active_plans.json to verify
    println!("# of results {}", tallies.len());
    for tally in tallies {
        for decrypted in tally.iter() {
            println!("result: {:?}", decrypted.tally_result);
        }
    }

    Ok(())
}
