mod arbitrary;
mod external;

pub use arbitrary::ArbitraryValidVotingTemplateGenerator;
pub use external::{ExternalValidVotingTemplateGenerator, TemplateLoadError};
use serde::{Deserialize, Serialize};
use vit_servicing_station_lib::db::models::proposals::ChallengeType;

#[derive(Serialize, Deserialize, Clone)]
pub struct FundTemplate {
    pub id: i32,
    pub goal: String,
    pub rewards_info: String,
    pub threshold: Option<u32>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalTemplate {
    pub internal_id: String,
    pub category_name: String,
    pub proposal_id: String,
    pub proposal_title: String,
    #[serde(default)]
    pub proposal_summary: String,
    pub proposal_problem: String,
    #[serde(default)]
    pub proposal_solution: String,
    pub proposal_funds: String,
    pub proposal_url: String,
    pub proposal_impact_score: String,
    #[serde(default)]
    pub files_url: String,
    pub proposer_name: String,
    #[serde(default)]
    pub proposer_url: String,
    #[serde(default)]
    pub proposer_relevant_experience: String,
    pub chain_vote_options: String,
    pub chain_vote_type: String,
    pub challenge_id: Option<String>,
}

impl ProposalTemplate {
    pub fn proposal_impact_score_as_integer(&self) -> i64 {
        (self.proposal_impact_score_as_float() * 100f64) as i64
    }

    pub fn proposal_impact_score_as_float(&self) -> f64 {
        serde_json::from_str(&self.proposal_impact_score).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChallengeTemplate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub rewards_total: String,
    pub challenge_url: String,
    pub fund_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalChallengeInfoTemplate {
    pub id: i32,
    pub challenge_id: i32,
    pub challenge_type: ChallengeType,
    pub proposal_solution: Option<String>,
    pub proposal_brief: Option<String>,
    pub proposal_importance: Option<String>,
    pub proposal_goal: Option<String>,
    pub proposal_metrics: Option<String>,
}

pub trait ValidVotingTemplateGenerator {
    fn next_proposal(&mut self) -> ProposalTemplate;
    fn next_challenge(&mut self) -> ChallengeTemplate;
    fn next_fund(&mut self) -> FundTemplate;
    fn next_proposal_challenge_info(&mut self) -> ProposalChallengeInfoTemplate;
}
