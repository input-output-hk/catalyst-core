pub mod context;
pub mod endpoints;

use warp::{Filter, Rejection, Reply};

pub async fn filter(
    ctx: context::SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let root = warp::path!("api" / "v0" / ..);
    endpoints::filter(root.boxed(), ctx).await
}
