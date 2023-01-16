use super::handlers::*;
use crate::rest::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let submit_tx_filter = warp::path!("tx")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context)
        .and_then(submit_tx)
        .boxed();

    root.and(submit_tx_filter).boxed()
}
