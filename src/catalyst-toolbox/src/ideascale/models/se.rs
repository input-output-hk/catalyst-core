use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Challenge {
    pub challenge_type: String,
    pub challenge_url: String,
    pub description: String,
    pub fund_id: String,
    pub id: String,
    pub rewards_total: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct Fund {
    pub id: i32,
    pub goal: String,
    pub threshold: i64,
}

#[derive(Debug, Serialize)]
pub struct Proposal {
    pub category_name: String,
    #[serde(default = "default_vote_options")]
    pub chain_vote_options: String,
    pub chain_vote_type: String,
    pub internal_id: String,
    pub proposal_funds: String,
    pub proposal_id: String,
    pub proposal_impact_score: String,
    pub proposal_solution: String,
    pub proposal_summary: String,
    pub proposal_title: String,
    pub proposal_url: String,
    pub proposer_email: String,
    pub proposer_name: String,
    pub proposer_relevant_experience: String,
    #[serde(default)]
    pub proposer_url: String,
}

#[allow(dead_code)]
fn default_vote_options() -> &'static str {
    "blank,yes,no"
}
