use super::handlers::*;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let from_voteplan_id_and_caster = warp::path::end()
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context)
        .and_then(get_vote_by_caster_and_voteplan_id);

    root.and(from_voteplan_id_and_caster)
}
