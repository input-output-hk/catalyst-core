mod ids;
use crate::v0::context::SharedContext;

use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(context: SharedContext) -> BoxedFilter<()> {
    let root = warp::path!("ids" / ..);
    root.and(ids::filter(context)).boxed()
}
