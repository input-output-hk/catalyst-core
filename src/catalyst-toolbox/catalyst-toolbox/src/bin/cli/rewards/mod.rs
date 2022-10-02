mod community_advisors;
mod dreps;
mod full;
mod proposers;
mod veterans;
mod voters;

use std::{collections::HashMap, path::PathBuf};

use catalyst_toolbox::{
    http::default_http_client,
    rewards::{proposers as proposers_lib, VoteCount},
};
use color_eyre::{eyre::eyre, Report};
use jormungandr_lib::{
    crypto::{account::Identifier, hash::Hash},
    interfaces::AccountVotes,
};
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

    /// Calculate full rewards based on a config file
    Full { path: PathBuf },

    /// Calculate rewards for propsers
    Proposers(proposers_lib::ProposerRewards),
}

impl Rewards {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Rewards::Voters(cmd) => cmd.exec(),
            Rewards::CommunityAdvisors(cmd) => cmd.exec(),
            Rewards::Veterans(cmd) => cmd.exec(),
            Rewards::Dreps(cmd) => cmd.exec(),
            Rewards::Full { path } => full::full_rewards(&path),
            Rewards::Proposers(proposers) => {
                proposers::rewards(&proposers, &default_http_client(None))
            }
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
