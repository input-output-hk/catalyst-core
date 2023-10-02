use crate::db::{
    models::community_advisors_reviews::AdvisorReview,
    queries::community_advisors_reviews as advisor_reviews_queries,
};
use crate::v0::context::SharedContext;
use crate::v0::endpoints::advisor_reviews::schemas::GroupedReviews;
use crate::v0::errors::HandleError;
use std::collections::HashMap;

pub async fn get_advisor_reviews_with_proposal_id(
    id: i32,
    context: SharedContext,
) -> Result<GroupedReviews, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    let reviews = advisor_reviews_queries::query_reviews_by_fund_id(id, pool).await?;
    Ok(group_reviews_by_assessor(reviews))
}

fn group_reviews_by_assessor(reviews: Vec<AdvisorReview>) -> GroupedReviews {
    let mut map: HashMap<String, Vec<AdvisorReview>> = HashMap::new();
    for review in reviews {
        map.entry(review.assessor.clone()).or_default().push(review);
    }
    GroupedReviews(map)
}
