use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::{Rewards, Threshold};
use catalyst_toolbox::utils::assert_are_close;

use color_eyre::eyre::eyre;
use color_eyre::{Report, Result};
use jcli_lib::block::open_output;
use jcli_lib::jcli_lib::block::Common;

use structopt::StructOpt;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VotersRewards {
    #[structopt(flatten)]
    common: Common,
    /// Reward (in LOVELACE) to be distributed
    #[structopt(long)]
    total_rewards: u64,

    /// Path to a json encoded list of `SnapshotInfo`
    #[structopt(long)]
    snapshot_info_path: PathBuf,

    /// Path to a json-encoded list of VotePlanStatusFull to consider for voters
    /// participation in the election.
    /// This can be retrived from the v1/vote/active/plans/full endpoint exposed
    /// by a Jormungandr node.
    #[structopt(long)]
    votes_count_path: PathBuf,

    /// Number of global votes required to be able to receive voter rewards
    #[structopt(long, default_value)]
    vote_threshold: usize,

    /// Path to a json-encoded map from challenge id to an optional required threshold
    /// per-challenge in order to receive rewards.
    #[structopt(long)]
    per_challenge_threshold: Option<PathBuf>,

    /// Path to the list of proposals active in this election.
    /// Can be obtained from /api/v0/proposals.
    #[structopt(long)]
    proposals: PathBuf,
}

fn write_rewards_results(
    common: &Option<PathBuf>,
    rewards: &BTreeMap<MainnetRewardAddress, Rewards>,
) -> Result<(), Report> {
    let writer = open_output(common)?;
    let header = ["Address", "Reward for the voter (lovelace)"];
    let mut csv_writer = csv::Writer::from_writer(writer);
    csv_writer.write_record(&header)?;

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
            per_challenge_threshold,
            proposals,
        } = self;

<<<<<<< HEAD
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
=======
        let proposals = serde_json::from_reader::<_, Vec<FullProposalInfo>>(
            jcli_lib::utils::io::open_file_read(&Some(proposals))?,
        )?;

        let vote_count = super::extract_individual_votes(
            proposals.clone(),
            serde_json::from_reader::<_, HashMap<Identifier, Vec<AccountVotes>>>(
                jcli_lib::utils::io::open_file_read(&Some(votes_count_path))?,
            )?,
        )?;

        let snapshot: Vec<SnapshotInfo> = serde_json::from_reader(
            jcli_lib::utils::io::open_file_read(&Some(snapshot_info_path))?,
        )?;

        let additional_thresholds: HashMap<i32, usize> = if let Some(file) = per_challenge_threshold
        {
            serde_json::from_reader(jcli_lib::utils::io::open_file_read(&Some(file))?)?
        } else {
            HashMap::new()
        };

        let results = calc_voter_rewards(
            vote_count,
            snapshot,
            Threshold::new(vote_threshold, additional_thresholds, proposals)?,
            Rewards::from(total_rewards),
        )?;

        let actual_rewards = results.values().sum::<Rewards>();
        assert_are_close(actual_rewards, Rewards::from(total_rewards));

        write_rewards_results(common, &results)?;
        Ok(())
>>>>>>> main
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
        vote_threshold,
        snapshot,
        Rewards::from(total_rewards),
    )?;

    let actual_rewards = results.values().sum::<Rewards>();
    assert_are_close(actual_rewards, Rewards::from(total_rewards));

    write_rewards_results(&Some(output.to_path_buf()), &results)?;
    Ok(())
}
