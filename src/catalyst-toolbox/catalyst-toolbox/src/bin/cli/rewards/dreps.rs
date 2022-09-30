use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::{Rewards, Threshold};
use color_eyre::Report;
use jcli_lib::jcli_lib::block::Common;
use jormungandr_lib::{crypto::account::Identifier, interfaces::AccountVotes};
use snapshot_lib::{registration::MainnetRewardAddress, SnapshotInfo};
use structopt::StructOpt;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct DrepsRewards {
    #[structopt(flatten)]
    common: Common,
    /// Reward (in dollars) to be distributed proportionally to delegated stake with respect to total stake.
    /// The total amount will only be awarded if dreps control all of the stake.
    #[structopt(long)]
    total_rewards: u64,

    /// Path to a json encoded list of `SnapshotInfo`
    #[structopt(long)]
    snapshot_info_path: PathBuf,

    /// Path to a json-encoded list of proposal every user has voted for.
    /// This can be retrived from the v1/account-votes-all endpoint exposed
    /// by a Jormungandr node.
    #[structopt(long)]
    votes_count_path: PathBuf,

    /// Number of global votes required to be able to receive voter rewards
    #[structopt(long, default_value)]
    vote_threshold: u64,

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
    common: Common,
    rewards: &BTreeMap<MainnetRewardAddress, Rewards>,
) -> Result<(), Report> {
    let writer = common.open_output()?;
    let header = ["Address", "Reward for the voter (lovelace)"];
    let mut csv_writer = csv::Writer::from_writer(writer);
    csv_writer.write_record(&header)?;

    for (address, rewards) in rewards.iter() {
        let record = [address.to_string(), rewards.trunc().to_string()];
        csv_writer.write_record(&record)?;
    }

    Ok(())
}

impl DrepsRewards {
    pub fn exec(self) -> Result<(), Report> {
        let DrepsRewards {
            common,
            total_rewards,
            snapshot_info_path,
            votes_count_path,
            vote_threshold,
            per_challenge_threshold,
            proposals,
        } = self;

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
            Threshold::new(
                vote_threshold
                    .try_into()
                    .expect("vote threshold is too big"),
                additional_thresholds,
                proposals,
            )?,
            Rewards::from(total_rewards),
        )?;

        write_rewards_results(common, &results)?;
        Ok(())
    }
}
