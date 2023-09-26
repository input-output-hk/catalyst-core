use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::{Rewards, Threshold};
use catalyst_toolbox::types::proposal::FullProposalInfo;
use catalyst_toolbox::utils::csv::dump_to_csv_or_print;
use clap::Parser;
use color_eyre::Report;
use jormungandr_lib::{crypto::account::Identifier, interfaces::AccountVotes};
use serde::Serialize;
use snapshot_lib::registration::RewardAddress;
use snapshot_lib::SnapshotInfo;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct DrepsRewards {
    /// Path to the output file
    /// print to stdout if not provided
    #[clap(long)]
    output: Option<PathBuf>,

    /// Reward (in dollars) to be distributed proportionally to delegated stake with respect to total stake.
    /// The total amount will only be awarded if dreps control all of the stake.
    #[clap(long)]
    total_rewards: u64,

    /// Path to a json encoded list of `SnapshotInfo`
    #[clap(long)]
    snapshot_info_path: PathBuf,

    /// Path to a json-encoded list of proposal every user has voted for.
    /// This can be retrived from the v1/account-votes-all endpoint exposed
    /// by a Jormungandr node.
    #[clap(long)]
    votes_count_path: PathBuf,

    /// Number of global votes required to be able to receive voter rewards
    #[clap(long, default_value = "0")]
    vote_threshold: u64,

    /// Path to a json-encoded map from challenge id to an optional required threshold
    /// per-challenge in order to receive rewards.
    #[clap(long)]
    per_challenge_threshold: Option<PathBuf>,

    /// Path to the list of proposals active in this election.
    /// Can be obtained from /api/v0/proposals.
    #[clap(long)]
    proposals: PathBuf,
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

impl DrepsRewards {
    pub fn exec(self) -> Result<(), Report> {
        let DrepsRewards {
            output,
            total_rewards,
            snapshot_info_path,
            votes_count_path,
            vote_threshold,
            per_challenge_threshold,
            proposals,
            ..
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

        write_rewards_results(&output, results)?;
        Ok(())
    }
}
