use super::handlers::*;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let reviews = warp::path!(i32)
        .and(warp::get())
        .and(with_context)
        .and_then(get_reviews_with_proposal_id);

    root.and(reviews)
}
