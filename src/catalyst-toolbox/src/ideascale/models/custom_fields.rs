use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CustomFieldTags {
    pub proposer_url: String,
    pub proposal_solution: String,
    pub proposal_brief: String,
    pub proposal_importance: String,
    pub proposal_goal: String,
    pub proposal_metrics: String,
}
