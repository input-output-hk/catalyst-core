use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Challenge {
    challenge_type: String,
    challenge_url: String,
    description: String,
    fund_id: String,
    id: String,
    proposers_rewards: String,
    rewards_total: String,
    title: String,
}

#[derive(Debug, Serialize)]
pub struct Fund {
    id: i64,
    goal: String,
    rewards_info: String,
    threshold: i64,
}

#[derive(Debug, Serialize)]
pub struct Proposal {
    category_name: String,
    #[serde(default = "default_vote_options")]
    chain_vote_options: String,
    chain_vote_type: String,
    internal_id: String,
    proposal_funds: String,
    proposal_id: String,
    proposal_impact_score: String,
    proposal_solution: String,
    proposal_summary: String,
    proposal_title: String,
    proposal_url: String,
    proposer_email: String,
    proposer_name: String,
    proposer_relevant_experience: String,
    #[serde(default)]
    proposer_url: String,
}

fn default_vote_options() -> &'static str {
    "blank,yes,no"
}
