use crate::db::{
    models::{challenges::Challenge, proposals::Proposal},
    queries::challenges as challenges_queries,
};
use crate::v0::context::SharedContext;
use crate::v0::errors::HandleError;

pub async fn get_all_challenges(context: SharedContext) -> Result<Vec<Challenge>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    challenges_queries::query_all_challenges(&pool).await
}

pub async fn get_challenge_by_id(
    id: i32,
    context: SharedContext,
) -> Result<Challenge, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    challenges_queries::query_challenge_by_id(id, &pool).await
}

pub async fn get_challenge_proposals_by_id(
    id: i32,
    context: SharedContext,
) -> Result<Vec<Proposal>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    challenges_queries::query_challenge_proposals_by_id(id, &pool).await
}
