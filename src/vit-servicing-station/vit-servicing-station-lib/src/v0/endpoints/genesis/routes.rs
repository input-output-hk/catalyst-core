use super::handlers::get_genesis;
use crate::v0::context::SharedContext;
use crate::v0::endpoints::genesis::handlers::get_genesis_for_fund;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let default_block0 = warp::path::end()
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_genesis)
        .boxed();

    let block0_per_fund = warp::path!(i32)
        .and(warp::get())
        .and(with_context)
        .and_then(get_genesis_for_fund)
        .boxed();

    root.and(default_block0.or(block0_per_fund)).boxed()
}
