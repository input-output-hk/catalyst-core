use std::{
    fs::File,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use serde::Deserialize;
use serde_json::from_reader;

pub(super) fn full_rewards(path: &Path) -> Result<()> {
    let config = from_reader(File::open(path)?)?;
    let Config {
        inputs:
            Inputs {
                block_file,
                vote_count_path,
                snapshot_path,
            },
        params:
            Params {
                registration_threshold,
                vote_threshold,
                total_rewards,
            },
    } = config;

    super::voters::voter_rewards(
        Some(block_file),
        vote_count_path,
        snapshot_path,
        registration_threshold,
        vote_threshold,
        total_rewards,
    )?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Config {
    inputs: Inputs,
    params: Params,
}

#[derive(Debug, Deserialize)]
struct Inputs {
    block_file: PathBuf,
    vote_count_path: PathBuf,
    snapshot_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Params {
    registration_threshold: u64,
    vote_threshold: u64,
    total_rewards: u64,
}
