use super::handlers::*;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let convert_root = warp::path!("convert" / ..);

    let id = warp::path!("id-from-hash" / String)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_id_from_hash)
        .boxed();

    let hash = warp::path!("hash-from-id" / String)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_hash_from_id)
        .boxed();

    root.and(convert_root.and(id.or(hash))).boxed()
}
