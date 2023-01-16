use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::{Rewards, Threshold, VoteCount};
use catalyst_toolbox::utils::assert_are_close;

use color_eyre::eyre::eyre;
use color_eyre::{Report, Result};
use jcli_lib::block::open_output;
use jcli_lib::jcli_lib::block::Common;

use clap::Parser;
use snapshot_lib::registration::MainnetRewardAddress;
use snapshot_lib::SnapshotInfo;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct VotersRewards {
    #[clap(flatten)]
    common: Common,
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
    common: &Option<PathBuf>,
    rewards: &BTreeMap<MainnetRewardAddress, Rewards>,
) -> Result<(), Report> {
    let writer = open_output(common)?;
    let header = ["Address", "Reward for the voter (lovelace)"];
    let mut csv_writer = csv::Writer::from_writer(writer);
    csv_writer.write_record(header)?;

    for (address, rewards) in rewards.iter() {
        let record = [address.to_string(), rewards.trunc().to_string()];
        csv_writer.write_record(&record)?;
    }

    Ok(())
}

impl VotersRewards {
    pub fn exec(self) -> Result<(), Report> {
        let VotersRewards {
            common,
            total_rewards,
            snapshot_info_path,
            votes_count_path,
            vote_threshold,
        } = self;

        voter_rewards(
            common
                .output_file
                .as_deref()
                .ok_or(eyre!("missing output file"))?,
            &votes_count_path,
            &snapshot_info_path,
            vote_threshold,
            total_rewards,
        )
    }
}

pub fn voter_rewards(
    output: &Path,
    votes_count_path: &Path,
    snapshot_path: &Path,
    vote_threshold: u64,
    total_rewards: u64,
) -> Result<()> {
    let vote_count: VoteCount = serde_json::from_reader(jcli_lib::utils::io::open_file_read(
        &Some(votes_count_path),
    )?)?;

    let snapshot: Vec<SnapshotInfo> =
        serde_json::from_reader(jcli_lib::utils::io::open_file_read(&Some(snapshot_path))?)?;

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

    write_rewards_results(&Some(output.to_path_buf()), &results)?;
    Ok(())
}
