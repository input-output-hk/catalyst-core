use crate::rest::v0::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub mod control;
pub mod data;
pub mod submit;

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let control_root = warp::path!("control" / ..);
    let control_filter = control::filter(control_root.boxed(), context.clone()).await;

    let data_root = warp::path!("data" / ..);
    let data_filter = data::filter(data_root.boxed(), context.clone()).await;

    let submit_root = warp::path!("submit" / ..);
    let submit_filter = submit::filter(submit_root.boxed(), context.clone()).await;

    root.and(control_filter.or(data_filter).or(submit_filter))
        .boxed()
}
