use crate::db::schema::proposal_simple_challenge;
use diesel::ExpressionMethods;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ChallengeInfo {
    #[serde(alias = "proposalSolution")]
    pub proposal_solution: String,
}

pub type ChallengeSqlValues = (
    diesel::dsl::Eq<proposal_simple_challenge::proposal_id, String>,
    diesel::dsl::Eq<proposal_simple_challenge::proposal_solution, String>,
);

impl ChallengeInfo {
    pub fn to_sql_values_with_proposal_id(&self, proposal_id: &str) -> ChallengeSqlValues {
        (
            proposal_simple_challenge::proposal_id.eq(proposal_id.to_string()),
            proposal_simple_challenge::proposal_solution.eq(self.proposal_solution.clone()),
        )
    }
}
