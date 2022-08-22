mod community_advisors;
mod dreps;
mod veterans;
mod voters;

use catalyst_toolbox::rewards::VoteCount;
use color_eyre::{eyre::eyre, Report};
use jormungandr_lib::{
    crypto::{account::Identifier, hash::Hash},
    interfaces::AccountVotes,
};
use std::collections::HashMap;
use structopt::StructOpt;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Rewards {
    /// Calculate rewards for voters base on their stake
    Voters(voters::VotersRewards),

    /// Calculate rewards for dreps based on their delegated stake
    Dreps(dreps::DrepsRewards),

    /// Calculate community advisors rewards
    CommunityAdvisors(community_advisors::CommunityAdvisors),

    /// Calculate rewards for veteran community advisors
    Veterans(veterans::VeteransRewards),
}

impl Rewards {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Rewards::Voters(cmd) => cmd.exec(),
            Rewards::CommunityAdvisors(cmd) => cmd.exec(),
            Rewards::Veterans(cmd) => cmd.exec(),
            Rewards::Dreps(cmd) => cmd.exec(),
        }
    }
}

fn extract_individual_votes(
    proposals: Vec<FullProposalInfo>,
    votes: HashMap<Identifier, Vec<AccountVotes>>,
) -> Result<VoteCount, Report> {
    let proposals_per_voteplan =
        proposals
            .into_iter()
            .fold(<HashMap<_, Vec<_>>>::new(), |mut acc, prop| {
                let entry = acc
                    .entry(prop.voteplan.chain_voteplan_id.clone())
                    .or_default();
                entry.push(prop);
                entry.sort_by_key(|p| p.voteplan.chain_proposal_index);
                acc
            });

    votes
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
        })
}
