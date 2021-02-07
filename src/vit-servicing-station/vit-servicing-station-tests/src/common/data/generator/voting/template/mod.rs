mod arbitrary;
mod external;

pub use arbitrary::ArbitraryValidVotingTemplateGenerator;
pub use external::{ExternalValidVotingTemplateGenerator, TemplateLoadError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FundTemplate {
    pub id: i32,
    pub goal: String,
    pub rewards_info: String,
    pub threshold: Option<u32>,
}
#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct ChallengeTemplate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub rewards_total: String,
    pub challenge_url: String,
    pub fund_id: Option<String>,
}

pub trait ValidVotingTemplateGenerator {
    fn next_proposal(&mut self) -> ProposalTemplate;
    fn next_challenge(&mut self) -> ChallengeTemplate;
    fn next_fund(&mut self) -> FundTemplate;
}
