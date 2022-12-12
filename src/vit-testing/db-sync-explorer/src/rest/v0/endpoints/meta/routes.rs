use super::handlers::*;
use crate::rest::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let meta_info = warp::path::end()
        .and(warp::get())
        .and(with_context)
        .and_then(get_meta_info)
        .boxed();

    root.and(meta_info).boxed()
}
