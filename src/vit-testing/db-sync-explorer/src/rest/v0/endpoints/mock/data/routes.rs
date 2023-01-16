use super::handlers::*;
use crate::rest::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let db_sync_content = warp::path!("content")
        .and(warp::get())
        .and(with_context)
        .and_then(get_db_sync_content)
        .boxed();

    root.and(db_sync_content).boxed()
}
