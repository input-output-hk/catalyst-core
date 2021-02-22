use crate::db::schema::proposal_community_choice_challenge;
use diesel::ExpressionMethods;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ChallengeInfo {
    #[serde(alias = "proposalBrief")]
    pub proposal_brief: String,
    #[serde(alias = "proposalImportance")]
    pub proposal_importance: String,
    #[serde(alias = "proposalGoal")]
    pub proposal_goal: String,
    #[serde(alias = "proposalMetrics")]
    pub proposal_metrics: String,
}

pub type ChallengeSqlValues = (
    diesel::dsl::Eq<proposal_community_choice_challenge::proposal_id, String>,
    diesel::dsl::Eq<proposal_community_choice_challenge::proposal_brief, String>,
    diesel::dsl::Eq<proposal_community_choice_challenge::proposal_importance, String>,
    diesel::dsl::Eq<proposal_community_choice_challenge::proposal_goal, String>,
    diesel::dsl::Eq<proposal_community_choice_challenge::proposal_metrics, String>,
);

impl ChallengeInfo {
    pub fn to_sql_values_with_proposal_id(&self, proposal_id: &str) -> ChallengeSqlValues {
        (
            proposal_community_choice_challenge::proposal_id.eq(proposal_id.to_string()),
            proposal_community_choice_challenge::proposal_brief.eq(self.proposal_brief.clone()),
            proposal_community_choice_challenge::proposal_importance
                .eq(self.proposal_importance.clone()),
            proposal_community_choice_challenge::proposal_goal.eq(self.proposal_goal.clone()),
            proposal_community_choice_challenge::proposal_metrics.eq(self.proposal_metrics.clone()),
        )
    }
}
