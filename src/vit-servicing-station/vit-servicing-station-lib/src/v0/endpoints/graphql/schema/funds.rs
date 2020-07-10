use crate::db;
use crate::db::{
    models::{funds::Fund, voteplans::Voteplan},
    queries::voteplans as voteplans_queries,
};
use crate::utils::datetime::unix_timestamp_to_datetime;
use async_graphql::Context;

#[async_graphql::Object]
impl Fund {
    #[field(desc = "Fund ID")]
    pub async fn id(&self) -> i32 {
        self.id
    }

    #[field(desc = "Fund Name")]
    pub async fn fund_name(&self) -> &str {
        &self.fund_name
    }

    #[field(desc = "Fund Goal")]
    pub async fn fund_goal(&self) -> &str {
        &self.fund_goal
    }

    #[field(desc = "Fund voting information")]
    pub async fn voting_power_info(&self) -> &str {
        &self.voting_power_info
    }

    #[field(desc = "Fund rewards information")]
    pub async fn rewards_info(&self) -> &str {
        &self.rewards_info
    }

    #[field(desc = "Fund start time, rfc3339 formatted")]
    pub async fn fund_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.fund_start_time).to_rfc3339()
    }

    #[field(desc = "Fund end time, rfc3339 formatted")]
    pub async fn fund_end_time(&self) -> String {
        unix_timestamp_to_datetime(self.fund_end_time).to_rfc3339()
    }

    #[field(desc = "Next fund start time, rfc3339 formatted")]
    pub async fn next_fund_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.next_fund_start_time).to_rfc3339()
    }

    #[field(desc = "Fund chain voteplans")]
    pub async fn chain_vote_plans(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::FieldResult<Vec<Voteplan>> {
        let pool = ctx.data::<db::DBConnectionPool>().unwrap();
        voteplans_queries::query_voteplan_by_id(self.id, pool)
            .await
            .map_err(async_graphql::FieldError::from)
    }
}
