use crate::db;
use crate::db::{
    models::{funds::Fund, voteplans::Voteplan},
    queries::voteplans as voteplans_queries,
};
use crate::utils::datetime::unix_timestamp_to_datetime;
use async_graphql::Context;

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

    pub async fn fund_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.fund_start_time).to_rfc3339()
    }

    pub async fn fund_end_time(&self) -> String {
        unix_timestamp_to_datetime(self.fund_end_time).to_rfc3339()
    }

    pub async fn next_fund_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.next_fund_start_time).to_rfc3339()
    }

    pub async fn chain_vote_plans(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::FieldResult<Vec<Voteplan>> {
        let pool = ctx.data::<db::DBConnectionPool>();
        voteplans_queries::query_voteplan_by_id(self.id, pool)
            .await
            .map_err(async_graphql::FieldError::from)
    }
}
