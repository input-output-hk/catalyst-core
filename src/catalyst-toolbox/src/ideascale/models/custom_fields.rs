use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CustomFieldTags {
    pub proposer_url: String,
    pub proposal_solution: String,
    pub proposal_brief: String,
    pub proposal_importance: String,
    pub proposal_goal: String,
    pub proposal_metrics: String,
    pub proposal_public_key: String,
    pub proposal_funds: String,
    pub proposal_relevant_experience: String,
    pub proposal_why: String,
}

impl Default for CustomFieldTags {
    fn default() -> Self {
        Self {
            proposer_url: "website_github_repository__not_required_".to_string(),
            proposal_solution: "problem_solution".to_string(),
            proposal_brief: "challenge_brief".to_string(),
            proposal_importance: "importance".to_string(),
            proposal_goal: "how_does_success_look_like_".to_string(),
            proposal_metrics: "key_metrics_to_measure".to_string(),
            proposal_public_key: "ada_payment_address".to_string(),
            proposal_funds: "requested_funds".to_string(),
            proposal_relevant_experience: "relevant_experience".to_string(),
            proposal_why: "importance".to_string(),
        }
    }
}
