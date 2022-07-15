use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::v0::context::SharedContext;

use super::logic;

pub async fn search_filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    root.and(
        warp::post()
            .and(warp::body::json())
            .and(warp::any().map(move || context.clone()))
            .and_then(logic::search),
    )
}

pub async fn search_count_filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    root.and(
        warp::post()
            .and(warp::body::json())
            .and(warp::any().map(move || context.clone()))
            .and_then(logic::search_count),
    )
}
