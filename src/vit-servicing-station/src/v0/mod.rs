pub mod context;
pub mod endpoints;

use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    ctx: context::SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let root = warp::path!("v0" / ..);
    endpoints::filter(root.boxed(), ctx)
}
