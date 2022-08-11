use catalyst_toolbox::rewards::voters::{calc_voter_rewards, Rewards, Threshold, VoteCount};
use catalyst_toolbox::utils::assert_are_close;
use jormungandr_lib::{
    crypto::{account::Identifier, hash::Hash},
    interfaces::AccountVotes,
};
use snapshot_lib::{registration::MainnetRewardAddress, SnapshotInfo};
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

use color_eyre::{eyre::eyre, Report};
use jcli_lib::jcli_lib::block::Common;
use structopt::StructOpt;

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

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

        let proposals_per_voteplan = serde_json::from_reader::<_, Vec<FullProposalInfo>>(
            jcli_lib::utils::io::open_file_read(&Some(proposals))?,
        )?
        .into_iter()
        .fold(<HashMap<_, Vec<_>>>::new(), |mut acc, prop| {
            let entry = acc
                .entry(prop.voteplan.chain_voteplan_id.clone())
                .or_default();
            entry.push(prop);
            entry.sort_by_key(|p| p.voteplan.chain_proposal_index);
            acc
        });

        let vote_count = serde_json::from_reader::<_, HashMap<Identifier, Vec<AccountVotes>>>(
            jcli_lib::utils::io::open_file_read(&Some(votes_count_path))?,
        )?
        .into_iter()
        .try_fold(VoteCount::new(), |mut acc, (account, votes)| {
            for vote in &votes {
                let voteplan = vote.vote_plan_id;
                let props = proposals_per_voteplan
                    .get(&voteplan.to_string())
                    .iter()
                    .flat_map(|p| p.iter())
                    .enumerate()
                    .filter(|(i, _p)| vote.votes.contains(&(*i as u8)))
                    .map(|(_, p)| {
                        Ok::<_, Report>(Hash::from(
                            <[u8; 32]>::try_from(p.proposal.chain_proposal_id.clone()).map_err(
                                |v| eyre!("Invalid proposal hash length {}, expected 32", v.len()),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                acc.entry(account.clone()).or_default().extend(props);
            }
            Ok::<_, Report>(acc)
        })?;

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
                vote_threshold,
                additional_thresholds,
                proposals_per_voteplan
                    .into_iter()
                    .flat_map(|(_k, v)| v.into_iter())
                    .collect(),
            )?,
            Rewards::from(total_rewards),
        )?;

        let actual_rewards = results.values().sum::<Rewards>();
        assert_are_close(actual_rewards, Rewards::from(total_rewards));

        write_rewards_results(common, &results)?;
        Ok(())
    }
}
