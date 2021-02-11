use crate::db::schema::proposals_challenge_info;
use async_graphql::static_assertions::_core::fmt::Formatter;
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChallengeType {
    Simple,
    CommunityChoice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Queryable)]
pub struct ProposalChallengeInfo {
    pub id: i32,
    #[serde(alias = "challengeId")]
    pub challenge_id: i32,
    #[serde(alias = "challengeType")]
    pub challenge_type: ChallengeType,
    #[serde(
        alias = "proposalSolution",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub proposal_solution: Option<String>,
    #[serde(
        alias = "proposalBrief",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub proposal_brief: Option<String>,
    #[serde(
        alias = "proposalImportance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub proposal_importance: Option<String>,
    #[serde(
        alias = "proposalGoal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub proposal_goal: Option<String>,
    #[serde(
        alias = "proposalMetrics",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub proposal_metrics: Option<String>,
}

impl std::fmt::Display for ChallengeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // should be implemented and safe to unwrap here
        let repr = serde_json::to_string(&self).unwrap();
        write!(f, "{}", repr.trim_matches('"'))
    }
}

#[allow(clippy::type_complexity)]
impl Insertable<proposals_challenge_info::table> for ProposalChallengeInfo {
    type Values = (
        diesel::dsl::Eq<proposals_challenge_info::challenge_id, i32>,
        diesel::dsl::Eq<proposals_challenge_info::challenge_type, String>,
        diesel::dsl::Eq<proposals_challenge_info::proposal_solution, Option<String>>,
        diesel::dsl::Eq<proposals_challenge_info::proposal_brief, Option<String>>,
        diesel::dsl::Eq<proposals_challenge_info::proposal_importance, Option<String>>,
        diesel::dsl::Eq<proposals_challenge_info::proposal_goal, Option<String>>,
        diesel::dsl::Eq<proposals_challenge_info::proposal_metrics, Option<String>>,
    );

    fn values(self) -> Self::Values {
        (
            proposals_challenge_info::challenge_id.eq(self.challenge_id),
            proposals_challenge_info::challenge_type.eq(self.challenge_type.to_string()),
            proposals_challenge_info::proposal_solution.eq(self.proposal_solution),
            proposals_challenge_info::proposal_brief.eq(self.proposal_brief),
            proposals_challenge_info::proposal_importance.eq(self.proposal_importance),
            proposals_challenge_info::proposal_goal.eq(self.proposal_goal),
            proposals_challenge_info::proposal_metrics.eq(self.proposal_metrics),
        )
    }
}
