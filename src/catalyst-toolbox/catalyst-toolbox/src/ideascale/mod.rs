mod fetch;
mod models;

use crate::http::HttpClient;
use crate::ideascale::models::de::{clean_str, Challenge, Fund, Funnel, Proposal, Stage};

use std::collections::{HashMap, HashSet};

use color_eyre::Report;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

pub use crate::ideascale::fetch::{Scores, Sponsors};
pub use crate::ideascale::models::custom_fields::CustomFieldTags;

// Id of funnel that do have rewards and should not count when importing funnels. It is static and
// should not change
const PROCESS_IMPROVEMENTS_ID: u32 = 7666;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error("invalid token")]
    InvalidToken,

    #[error(transparent)]
    Other(#[from] Report),
}

#[derive(Debug)]
pub struct IdeaScaleData {
    pub funnels: HashMap<u32, Funnel>,
    pub fund: Fund,
    pub challenges: HashMap<u32, Challenge>,
    pub proposals: Vec<Proposal>,
}

pub fn fetch_all(
    fund: usize,
    stage_label: &str,
    stages_filters: &[&str],
    excluded_proposals: &HashSet<u32>,
    client: &impl HttpClient,
) -> Result<IdeaScaleData, Error> {
    if !fetch::is_token_valid(client)? {
        return Err(Error::InvalidToken);
    }

    let funnels = fetch::get_funnels_data_for_fund(client)?
        .into_iter()
        .map(|f| (f.id, f))
        .collect();
    let funds = fetch::get_funds_data(client)?;

    let challenges: Vec<Challenge> = funds
        .iter()
        .filter(|f| f.id != PROCESS_IMPROVEMENTS_ID)
        .flat_map(|f| f.challenges.iter().cloned())
        .filter(|c| c.rewards > 0.into())
        .collect();

    let proposals: Vec<_> = challenges
        .par_iter()
        .map(|c| (fetch::get_proposals_data(client, c.id)))
        .collect();

    let matches = regex::Regex::new(&stages_filters.join("|"))?;
    let mut proposals: Vec<Proposal> = proposals
        .into_iter()
        // forcefully unwrap to pop errors directly
        // TODO: Handle error better here
        .flat_map(Result::unwrap)
        // filter out non approved or staged proposals
        .filter(|p| p.approved.as_bool() && filter_proposal_by_stage_type(&p.stage_type, &matches))
        .filter(|p| !excluded_proposals.contains(&p.proposal_id))
        .collect();

    proposals.sort_by_key(|p| p.proposal_id);

    let mut stages: Vec<_> = fetch::get_stages(client)?;
    stages.retain(|stage| filter_stages(stage, stage_label, &funnels));

    Ok(IdeaScaleData {
        funnels,
        fund: funds
            .into_iter()
            .find(|f| f.name.as_ref().contains(&format!("Fund {}", fund)))
            .unwrap_or_else(|| panic!("Selected fund {}, wasn't among the available funds", fund)),
        challenges: challenges
            .into_iter()
            .enumerate()
            .map(|(idx, c)| ((idx + 1) as u32, c))
            .collect(),
        proposals,
    })
}

pub fn build_fund(fund: i32, goal: String, threshold: i64) -> Vec<models::se::Fund> {
    vec![models::se::Fund {
        id: fund,
        goal,
        threshold,
        rewards_info: "".to_string(),
    }]
}

pub fn build_challenges(
    fund: i32,
    ideascale_data: &IdeaScaleData,
    sponsors: Sponsors,
) -> HashMap<u32, models::se::Challenge> {
    let funnels = &ideascale_data.funnels;

    ideascale_data
        .challenges
        .iter()
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
                    proposers_rewards: c.rewards.to_string(),
                    title: c.title.clone(),
                    highlight: sponsors.get(&c.challenge_url).map(|sponsor| {
                        models::se::Highlight {
                            sponsor: sponsor.clone(),
                        }
                    }),
                },
            )
        })
        .collect()
}

pub fn build_proposals(
    ideascale_data: &IdeaScaleData,
    built_challenges: &HashMap<u32, models::se::Challenge>,
    scores: &Scores,
    chain_vote_type: &str,
    fund: usize,
    tags: &CustomFieldTags,
) -> Vec<models::se::Proposal> {
    ideascale_data
        .proposals
        .iter()
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
                internal_id: i.to_string(),
                // this may change to an integer type in the future, would have to get from json value as so
                proposal_funds: get_from_extra_fields_options(
                    &p.custom_fields.fields,
                    &tags.proposal_funds,
                )
                .unwrap_or_default(),
                proposal_id: p.proposal_id.to_string(),
                proposal_impact_score: scores
                    .get(&p.proposal_id)
                    .cloned()
                    // it comes in the range of 0-5.0 => make it to the range of 0-500
                    .map(|v| (v * 100f32).trunc() as u32)
                    .unwrap_or(0u32)
                    .to_string(),
                proposal_summary: p.proposal_summary.to_string(),
                proposal_title: p.proposal_title.to_string(),
                proposal_url: p.proposal_url.to_string(),
                proposer_email: p.proposer.contact.clone(),
                proposer_name: p.proposer.name.clone(),
                proposer_relevant_experience: get_from_extra_fields(
                    &p.custom_fields.fields,
                    &tags.proposal_relevant_experience,
                )
                .map(|s| clean_str(&s))
                .unwrap_or_default(),
                proposer_url: get_from_extra_fields(&p.custom_fields.fields, &tags.proposer_url)
                    .unwrap_or_default(),
                proposal_solution: get_from_extra_fields(
                    &p.custom_fields.fields,
                    &tags.proposal_solution,
                ),
                proposal_brief: get_from_extra_fields(
                    &p.custom_fields.fields,
                    &tags.proposal_brief,
                ),
                proposal_importance: get_from_extra_fields(
                    &p.custom_fields.fields,
                    &tags.proposal_importance,
                )
                .map(|s| clean_str(&s)),
                proposal_goal: get_from_extra_fields(&p.custom_fields.fields, &tags.proposal_goal),
                proposal_metrics: get_from_extra_fields(
                    &p.custom_fields.fields,
                    &tags.proposal_metrics,
                ),
            }
        })
        .collect()
}

fn filter_proposal_by_stage_type(stage: &str, re: &Regex) -> bool {
    re.is_match(stage)
}

fn filter_stages(stage: &Stage, stage_label: &str, funnel_ids: &HashMap<u32, Funnel>) -> bool {
    stage.label.to_ascii_lowercase() == stage_label && funnel_ids.contains_key(&stage.funnel_id)
}

fn get_from_extra_fields(fields: &serde_json::Value, tag: &str) -> Option<String> {
    fields
        .get(tag)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn get_from_extra_fields_options(fields: &serde_json::Value, tags: &[String]) -> Option<String> {
    tags.iter()
        .map(|tag| get_from_extra_fields(fields, tag))
        .find(|x| x.is_some())
        .unwrap_or_default()
}
