use crate::db;
use crate::db::{
    models::{funds::Fund, voteplans::Voteplan},
    schema::voteplans::dsl as voteplans_dsl,
};
use async_graphql::Context;
use diesel::{ExpressionMethods, RunQueryDsl};

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
        self.fund_start_time.to_rfc3339()
    }

    pub async fn fund_end_time(&self) -> String {
        self.fund_end_time.to_rfc3339()
    }

    pub async fn next_fund_start_time(&self) -> String {
        self.next_fund_start_time.to_rfc3339()
    }

    pub async fn chain_vote_plans(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::FieldResult<Vec<Voteplan>> {
        let db_conn = ctx
            .data::<db::DBConnectionPool>()
            .get()
            .map_err(async_graphql::FieldError::from)?;
        let id = self.id;
        tokio::task::spawn_blocking(move || {
            diesel::QueryDsl::filter(voteplans_dsl::voteplans, voteplans_dsl::fund_id.eq(id))
                .load::<Voteplan>(&db_conn)
                .map_err(async_graphql::FieldError::from)
        })
        .await?
        .map_err(async_graphql::FieldError::from)
    }
}
