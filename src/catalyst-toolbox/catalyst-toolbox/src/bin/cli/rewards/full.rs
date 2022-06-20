use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::Context, Result};
use jcli_lib::block::{Common, Input};
use serde::Deserialize;

use super::voters::VotersRewards;

#[derive(Debug, Deserialize)]
struct Config {
    inputs: ConfigInputs,
    params: Params,
}

#[derive(Debug, Deserialize)]
struct ConfigInputs {
    challenges_path: PathBuf,
    active_voteplans_path: PathBuf,
    proposals_path: PathBuf,
    excluded_proposals_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Params {
    total_rewards: u64,
    snapshot_path: PathBuf,
    registration_threshold: u64,
    votes_count_path: PathBuf,
    vote_threshold: u64,
}

pub fn full_rewards(config: &Path) -> Result<()> {
    let file = BufReader::new(File::open(config)?);
    let config = serde_json::from_reader(file)?;
    let Config {
        inputs:
            ConfigInputs {
                challenges_path,
                active_voteplans_path,
                proposals_path,
                excluded_proposals_path,
            },
        params,
    } = config;


    let voter_rewards = VotersRewards {
        common: Common {
            input: Input {
                input_file: 
            }

        }
    };

    todo!()
}
