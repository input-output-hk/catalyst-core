use super::handlers::*;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let from_id = warp::path!(i32)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_proposal)
        .boxed();

    let proposals = warp::path::end()
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_all_proposals)
        .boxed();

    let from_voteplan_id_and_indexes = warp::path::end()
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context)
        .and_then(get_proposals_by_voteplan_id_and_index);

    root.and(from_id.or(proposals).or(from_voteplan_id_and_indexes))
        .boxed()
}
