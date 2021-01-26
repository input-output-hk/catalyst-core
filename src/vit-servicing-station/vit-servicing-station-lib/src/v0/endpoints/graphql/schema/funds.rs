use crate::db;
use crate::db::{
    models::{challenges::Challenge, funds::Fund, voteplans::Voteplan},
    queries::{challenges as challenges_queries, voteplans as voteplans_queries},
};
use crate::utils::datetime::unix_timestamp_to_datetime;
use async_graphql::Context;

#[async_graphql::Object]
impl Fund {
    /// Fund ID
    pub async fn id(&self) -> i32 {
        self.id
    }

    /// Fund Name
    pub async fn fund_name(&self) -> &str {
        &self.fund_name
    }

    /// Fund Goal
    pub async fn fund_goal(&self) -> &str {
        &self.fund_goal
    }

    /// Fund voting information
    pub async fn voting_power_info(&self) -> &str {
        &self.voting_power_info
    }

    /// Fund voting threshold
    pub async fn voting_power_threshold(&self) -> i64 {
        self.voting_power_threshold
    }

    /// Fund rewards information
    pub async fn rewards_info(&self) -> &str {
        &self.rewards_info
    }

    /// Fund start time, rfc3339 formatted
    pub async fn fund_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.fund_start_time).to_rfc3339()
    }

    /// Fund end time, rfc3339 formatted
    pub async fn fund_end_time(&self) -> String {
        unix_timestamp_to_datetime(self.fund_end_time).to_rfc3339()
    }

    /// Next fund start time, rfc3339 formatted
    pub async fn next_fund_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.next_fund_start_time).to_rfc3339()
    }

    /// Fund chain voteplans
    pub async fn chain_vote_plans(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::FieldResult<Vec<Voteplan>> {
        let pool = ctx.data::<db::DBConnectionPool>().unwrap();
        voteplans_queries::query_voteplan_by_id(self.id, pool)
            .await
            .map_err(async_graphql::FieldError::from)
    }

    /// Fund challenges
    pub async fn challenges(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::FieldResult<Vec<Challenge>> {
        let pool = ctx.data::<db::DBConnectionPool>().unwrap();
        println!("requesting challenges for fund id {}", self.id);
        challenges_queries::query_challenges_by_fund_id(self.id, pool)
            .await
            .map_err(async_graphql::FieldError::from)
    }
}
