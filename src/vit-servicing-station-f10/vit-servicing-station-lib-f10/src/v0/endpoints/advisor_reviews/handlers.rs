use super::logic;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn get_reviews_with_proposal_id(
    id: i32,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        logic::get_advisor_reviews_with_proposal_id(id, context).await,
    ))
}
