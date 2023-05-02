use crate::v0::context::SharedContext;

use super::handlers::{get_delegator_info, get_voters_info};
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let get_voters_info = warp::path!("voter" / String / String)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_voters_info);

    let get_delegator_info = warp::path!("delegator" / String / String)
        .and(warp::get())
        .and(with_context)
        .and_then(get_delegator_info);

    root.and(get_voters_info.or(get_delegator_info))
}
