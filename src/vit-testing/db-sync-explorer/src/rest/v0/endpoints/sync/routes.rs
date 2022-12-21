use super::handlers::*;
use crate::rest::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let behind = warp::path!("behind")
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_interval_behind_now_filter)
        .boxed();

    let progress = warp::path!("progress")
        .and(warp::get())
        .and(with_context)
        .and_then(progress_filter);

    root.and(behind.or(progress)).boxed()
}
