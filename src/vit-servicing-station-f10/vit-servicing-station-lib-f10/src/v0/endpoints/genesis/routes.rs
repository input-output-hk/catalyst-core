use super::handlers::get_genesis;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let block0 = warp::path::end()
        .and(warp::get())
        .and(with_context)
        .and_then(get_genesis)
        .boxed();

    root.and(block0).boxed()
}
