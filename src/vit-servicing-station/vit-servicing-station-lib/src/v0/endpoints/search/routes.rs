use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use crate::{
    db::queries::search::{SearchColumn, SearchTable},
    v0::context::SharedContext,
};

use super::logic::{self, SearchSortQueryParams};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    root.and(
        warp::path!(SearchTable / SearchColumn / String)
            .and(warp::get())
            .and(warp::query().map(|sort: SearchSortQueryParams| sort))
            .and(warp::any().map(move || context.clone()))
            .and_then(logic::search),
    )
}
