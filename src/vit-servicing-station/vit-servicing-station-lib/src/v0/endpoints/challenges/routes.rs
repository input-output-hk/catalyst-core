use super::handlers::*;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let challenges = warp::path::end()
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_challenges);

    let challenge_by_id = warp::path!(i32)
        .and(warp::get())
        .and(with_context)
        .and_then(get_challenge_by_id);

    root.and(challenge_by_id.or(challenges))
}
