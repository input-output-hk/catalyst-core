mod ids;
use crate::v0::context::SharedContext;

use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let ids_root = warp::path!("ids" / ..);
    ids::filter(root.and(ids_root).boxed(), context)
}
