use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::{Rewards, Threshold};
use catalyst_toolbox::utils::csv::dump_to_csv_or_print;
use catalyst_toolbox::utils::{assert_are_close, json_from_file};
use clap::Parser;
use color_eyre::{Report, Result};
use serde::{Deserialize, Serialize};
use snapshot_lib::registration::MainnetRewardAddress;
use snapshot_lib::SnapshotInfo;
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

fn write_rewards_results<RewardAddressType>(
    output: &Option<PathBuf>,
    rewards: BTreeMap<RewardAddressType, Rewards>,
) -> Result<(), Report>
where
    RewardAddressType: Serialize + std::fmt::Debug + Serialize,
{
    #[derive(Serialize, Debug)]
    struct Entry<RewardAddressType> {
        #[serde(rename = "Address")]
        address: RewardAddressType,
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

        voter_rewards::<MainnetRewardAddress>(
            &output,
            &votes_count_path,
            &snapshot_info_path,
            vote_threshold,
            total_rewards,
        )
    }
}

pub fn voter_rewards<RewardAddressType>(
    output: &Option<PathBuf>,
    votes_count_path: &Path,
    snapshot_path: &Path,
    vote_threshold: u64,
    total_rewards: u64,
) -> Result<()>
where
    RewardAddressType: for<'a> Deserialize<'a>
        + Serialize
        + Clone
        + Default
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::fmt::Debug,
{
    let vote_count = json_from_file(votes_count_path)?;
    let snapshot: Vec<SnapshotInfo<RewardAddressType>> = json_from_file(snapshot_path)?;

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
