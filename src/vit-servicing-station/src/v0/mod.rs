pub mod context;
pub mod services;

use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

// pub fn filter(ctx: context::SharedContext) -> impl Filter<Extract = (), Error = Rejection> + Clone {
//     let root = warp::path!("v0" / ..);
//     root.and(services::filter(ctx)).boxed()
// }
