use super::handlers::service_version;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let service_version_filter = warp::path::end()
        .and(warp::get())
        .and(with_context)
        .and_then(service_version)
        .boxed();

    root.and(service_version_filter).boxed()
}
