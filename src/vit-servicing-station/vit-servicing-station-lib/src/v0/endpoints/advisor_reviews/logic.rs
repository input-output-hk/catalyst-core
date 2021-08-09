use crate::db::{
    models::community_advisors_reviews::AdvisorReview,
    queries::community_advisors_reviews as advisor_reviews_queries,
};
use crate::v0::context::SharedContext;
use crate::v0::errors::HandleError;

pub async fn get_advisor_reviews_with_proposal_id(
    id: i32,
    context: SharedContext,
) -> Result<Vec<AdvisorReview>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    advisor_reviews_queries::query_reviews_by_fund_id(id, pool).await
}
