mod fetch;
mod models;

use crate::ideascale::fetch::Scores;
use crate::ideascale::models::de::{Challenge, Fund, Funnel, Proposal};

use std::collections::{HashMap, HashSet};

const PROPOSER_URL_TAG: &str = "website_github_repository__not_required_";
const PROPOSAL_SOLUTION_TAG: &str = "proposal_solution";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FetchError(#[from] fetch::Error),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct IdeaScaleData {
    funnels: HashMap<i32, Funnel>,
    fund: Fund,
    challenges: HashMap<i32, Challenge>,
    proposals: HashMap<i32, Proposal>,
    scores: Scores,
}

pub type Rewards = HashMap<i32, i64>;

pub async fn fetch_all(fund: usize, api_token: String) -> Result<IdeaScaleData, Error> {
    let funnels_task = tokio::spawn(fetch::get_funnels_data_for_fund(api_token.clone()));
    let funds_task = tokio::spawn(fetch::get_funds_data(api_token.clone()));
    let funnels = funnels_task
        .await??
        .into_iter()
        .map(|f| (f.id, f))
        .collect();
    let funds = funds_task.await??;
    let challenges: Vec<Challenge> = funds
        .iter()
        .flat_map(|f| f.challenges.iter().cloned())
        .collect();
    let proposals_tasks: Vec<_> = challenges
        .iter()
        .map(|c| tokio::spawn(fetch::get_proposals_data(c.id, api_token.clone())))
        .collect();
    let proposals: Vec<Proposal> = futures::future::try_join_all(proposals_tasks)
        .await?
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    let stage_ids: HashSet<i32> = proposals.iter().map(|p| p.stage_id).collect();

    let scores_tasks: Vec<_> = stage_ids
        .iter()
        .map(|id| {
            tokio::spawn(fetch::get_assessments_scores_by_stage_id(
                *id,
                api_token.clone(),
            ))
        })
        .collect();

    let scores: Scores = futures::future::try_join_all(scores_tasks)
        .await?
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    Ok(IdeaScaleData {
        funnels,
        fund: funds
            .into_iter()
            .find(|f| f.name.contains(&format!("Fund{}", fund)))
            .unwrap_or_else(|| panic!("Selected fund {}, wasn't among the available funds", fund)),
        challenges: challenges.into_iter().map(|c| (c.id, c)).collect(),
        proposals: proposals.into_iter().map(|p| (p.proposal_id, p)).collect(),
        scores,
    })
}

pub fn build_fund(
    ideascale_data: &IdeaScaleData,
    threshold: i64,
    rewards_info: String,
) -> Vec<models::se::Fund> {
    vec![models::se::Fund {
        id: ideascale_data.fund.id,
        goal: ideascale_data.fund.name.clone(),
        rewards_info,
        threshold,
    }]
}

pub fn build_challenges(
    ideascale_data: &IdeaScaleData,
    rewards: &Rewards,
) -> Vec<models::se::Challenge> {
    let funnels = &ideascale_data.funnels;
    ideascale_data
        .challenges
        .values()
        .map(|c| models::se::Challenge {
            challenge_type: funnels
                .get(&c.funnel_id)
                .unwrap_or_else(|| panic!("A funnel with id {} wasn't found", c.funnel_id))
                .is_community()
                .then(|| "community-choice")
                .unwrap_or("simple")
                .to_string(),
            challenge_url: c.challenge_url.clone(),
            description: c.description.clone(),
            fund_id: c.fund_id.to_string(),
            id: c.id.to_string(),
            // TODO: proposers_rewards to be removed
            proposers_rewards: "".to_string(),
            rewards_total: rewards.get(&c.id).cloned().unwrap_or(0).to_string(),
            title: c.title.clone(),
        })
        .collect()
}

pub fn build_proposals(
    ideascale_data: &IdeaScaleData,
    chain_vote_type: &str,
    fund: usize,
) -> Vec<models::se::Proposal> {
    let scores = &ideascale_data.scores;
    ideascale_data
        .proposals
        .values()
        .enumerate()
        .map(|(i, p)| models::se::Proposal {
            category_name: format!("Fund{}", fund),
            chain_vote_options: "blank,yes,no".to_string(),
            chain_vote_type: chain_vote_type.to_string(),
            internal_id: i.to_string(),
            proposal_funds: p.custom_fields.proposal_funds.clone(),
            proposal_id: p.proposal_id.to_string(),
            proposal_impact_score: scores
                .get(&p.proposal_id)
                .cloned()
                .unwrap_or(0f32)
                .to_string(),
            proposal_solution: p
                .custom_fields
                .extra
                .get(PROPOSAL_SOLUTION_TAG)
                .map_or("", |s| s.as_str().unwrap_or(""))
                .to_string(),
            proposal_summary: p.proposal_summary.clone(),
            proposal_title: p.proposal_title.clone(),
            proposal_url: p.proposal_url.clone(),
            proposer_email: p.proposer.contact.clone(),
            proposer_name: p.proposer.name.clone(),
            proposer_relevant_experience: p.custom_fields.proposal_relevant_experience.clone(),
            proposer_url: p
                .custom_fields
                .extra
                .get(PROPOSER_URL_TAG)
                .map(|c| c.as_str().unwrap())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}
