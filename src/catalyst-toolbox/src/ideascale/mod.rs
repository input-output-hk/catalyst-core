mod fetch;
mod models;

use crate::ideascale::fetch::Scores;
use crate::ideascale::models::de::{Challenge, Fund, Funnel, Proposal, Stage};

use std::collections::HashMap;

const PROPOSER_URL_TAG: &str = "website_github_repository__not_required_";
const PROPOSAL_SOLUTION_TAG: &str = "problem_solution";

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
    pub funnels: HashMap<u32, Funnel>,
    pub fund: Fund,
    pub challenges: HashMap<u32, Challenge>,
    pub proposals: HashMap<u32, Proposal>,
    pub scores: Scores,
}

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
        // filter out non
        .filter(|p| filter_proposal_by_stage_type(&p.stage_type))
        .collect();

    let mut stages: Vec<_> = fetch::get_stages(api_token.clone()).await?;
    stages.retain(|stage| filter_stages(stage, &funnels));

    let scores_tasks: Vec<_> = stages
        .iter()
        .map(|stage| {
            tokio::spawn(fetch::get_assessments_score(
                stage.assessment_id,
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
            .find(|f| f.name.as_ref().contains(&format!("Fund{}", fund)))
            .unwrap_or_else(|| panic!("Selected fund {}, wasn't among the available funds", fund)),
        challenges: challenges.into_iter().map(|c| (c.id, c)).collect(),
        proposals: proposals.into_iter().map(|p| (p.proposal_id, p)).collect(),
        scores,
    })
}

pub fn build_fund(fund: i32, goal: String, threshold: i64) -> Vec<models::se::Fund> {
    vec![models::se::Fund {
        id: fund,
        goal,
        threshold,
    }]
}

pub fn build_challenges(
    fund: i32,
    ideascale_data: &IdeaScaleData,
) -> HashMap<u32, models::se::Challenge> {
    let funnels = &ideascale_data.funnels;
    ideascale_data
        .challenges
        .values()
        .enumerate()
        .map(|(i, c)| {
            (
                c.id,
                models::se::Challenge {
                    challenge_type: funnels
                        .get(&c.funnel_id)
                        .unwrap_or_else(|| panic!("A funnel with id {} wasn't found", c.funnel_id))
                        .is_community()
                        .then(|| "community-choice")
                        .unwrap_or("simple")
                        .to_string(),
                    challenge_url: c.challenge_url.clone(),
                    description: c.description.to_string(),
                    fund_id: fund.to_string(),
                    id: i.to_string(),
                    rewards_total: c.rewards.to_string(),
                    title: c.title.clone(),
                },
            )
        })
        .collect()
}

pub fn build_proposals(
    ideascale_data: &IdeaScaleData,
    built_challenges: &HashMap<u32, models::se::Challenge>,
    chain_vote_type: &str,
    fund: usize,
) -> Vec<models::se::Proposal> {
    let scores = &ideascale_data.scores;
    ideascale_data
        .proposals
        .values()
        .enumerate()
        .map(|(i, p)| {
            let challenge = &built_challenges.get(&p.challenge_id).unwrap_or_else(|| {
                panic!(
                    "Expected a challenge with id {} for proposal with id {}",
                    p.challenge_id, p.proposal_id
                )
            });
            models::se::Proposal {
                category_name: format!("Fund{}", fund),
                chain_vote_options: "blank,yes,no".to_string(),
                challenge_id: challenge.id.clone(),
                challenge_type: challenge.challenge_type.clone(),
                chain_vote_type: chain_vote_type.to_string(),
                internal_id: p.proposal_id.to_string(),
                proposal_funds: p.custom_fields.proposal_funds.clone(),
                proposal_id: i.to_string(),
                proposal_impact_score: scores
                    .get(&p.proposal_id)
                    .cloned()
                    .unwrap_or(0f32)
                    .to_string(),
                proposal_solution: p
                    .custom_fields
                    .extra
                    .get(PROPOSAL_SOLUTION_TAG)
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string(),
                proposal_summary: p.proposal_summary.to_string(),
                proposal_title: p.proposal_title.to_string(),
                proposal_url: p.proposal_url.to_string(),
                proposer_email: p.proposer.contact.clone(),
                proposer_name: p.proposer.name.clone(),
                proposer_relevant_experience: p
                    .custom_fields
                    .proposal_relevant_experience
                    .to_string(),
                proposer_url: p
                    .custom_fields
                    .extra
                    .get(PROPOSER_URL_TAG)
                    .map(|c| c.as_str().unwrap())
                    .unwrap_or("")
                    .to_string(),
            }
        })
        .collect()
}

fn filter_proposal_by_stage_type(stage: &str) -> bool {
    matches!(stage, "Governance phase" | "Assess QA")
}

fn filter_stages(stage: &Stage, funnel_ids: &HashMap<u32, Funnel>) -> bool {
    matches!(stage.label.as_str(), "Assess") && funnel_ids.contains_key(&stage.funnel_id)
}
