use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::{Rewards, Threshold};
use catalyst_toolbox::utils::csv::dump_to_csv_or_print;
use catalyst_toolbox::utils::{assert_are_close, json_from_file};
use clap::Parser;
use color_eyre::{Report, Result};
use serde::Serialize;
use snapshot_lib::registration::RewardAddress;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct VotersRewards {
    /// Path to the output file
    /// print to stdout if not provided
    #[clap(long)]
    output: Option<PathBuf>,

    /// Reward (in LOVELACE) to be distributed
    #[clap(long)]
    total_rewards: u64,

    /// Path to a json encoded list of `SnapshotInfo`
    #[clap(long)]
    snapshot_info_path: PathBuf,

    /// Path to a json-encoded list of VotePlanStatusFull to consider for voters
    /// participation in the election.
    /// This can be retrived from the v1/vote/active/plans/full endpoint exposed
    /// by a Jormungandr node.
    #[clap(long)]
    votes_count_path: PathBuf,

    /// Number of global votes required to be able to receive voter rewards
    #[clap(long, default_value = "0")]
    vote_threshold: u64,
}

fn write_rewards_results(
    output: &Option<PathBuf>,
    rewards: BTreeMap<RewardAddress, Rewards>,
) -> Result<(), Report> {
    #[derive(Serialize, Debug)]
    struct Entry {
        #[serde(rename = "Address")]
        address: RewardAddress,
        #[serde(rename = "Reward for the voter (lovelace)")]
        reward: Rewards,
    }

    dump_to_csv_or_print(
        output,
        rewards
            .into_iter()
            .map(|(address, reward)| Entry { address, reward }),
    )?;

    Ok(())
}

impl VotersRewards {
    pub fn exec(self) -> Result<(), Report> {
        let VotersRewards {
            output,
            total_rewards,
            snapshot_info_path,
            votes_count_path,
            vote_threshold,
        } = self;

        voter_rewards(
            &output,
            &votes_count_path,
            &snapshot_info_path,
            vote_threshold,
            total_rewards,
        )
    }
}

pub fn voter_rewards(
    output: &Option<PathBuf>,
    votes_count_path: &Path,
    snapshot_path: &Path,
    vote_threshold: u64,
    total_rewards: u64,
) -> Result<()> {
    let vote_count = json_from_file(votes_count_path)?;
    let snapshot = json_from_file(snapshot_path)?;

    let results = calc_voter_rewards(
        vote_count,
        snapshot,
        Threshold::new(
            vote_threshold as usize,
            Default::default(),
            Default::default(),
        )?,
        Rewards::from(total_rewards),
    )?;

    let actual_rewards = results.values().sum::<Rewards>();
    assert_are_close(actual_rewards, Rewards::from(total_rewards));

    write_rewards_results(output, results)?;
    Ok(())
}
