use crate::db::models::{funds::Fund, voteplans::Voteplan};

#[async_graphql::Object]
impl Fund {
    pub async fn id(&self) -> i32 {
        self.id
    }

    pub async fn fund_name(&self) -> &str {
        &self.fund_name
    }

    pub async fn fund_goal(&self) -> &str {
        &self.fund_goal
    }

    pub async fn voting_power_info(&self) -> &str {
        &self.voting_power_info
    }

    pub async fn rewards_info(&self) -> &str {
        &self.rewards_info
    }

    pub async fn fund_start_time(&self) -> &str {
        &self.fund_start_time
    }

    pub async fn fund_end_time(&self) -> &str {
        &self.fund_end_time
    }

    pub async fn next_fund_start_time(&self) -> &str {
        &self.next_fund_start_time
    }

    pub async fn chain_vote_plans(&self) -> Vec<Voteplan> {
        vec![]
    }
}
