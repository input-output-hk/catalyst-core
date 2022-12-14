use super::logic;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn get_challenges(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_all_challenges(context).await))
}

pub async fn get_challenge_by_id(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_challenge_by_id(id, context).await))
}

pub async fn get_challenge_by_id_and_group_id(
    id: i32,
    voter_group_id: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        logic::get_challenge_by_id_and_group_id(id, voter_group_id, context).await,
    ))
}
