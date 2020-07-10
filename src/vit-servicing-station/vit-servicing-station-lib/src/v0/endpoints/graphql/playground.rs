use async_graphql::http::GraphQLPlaygroundConfig;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

#[allow(dead_code)]
pub fn filter(
    root: BoxedFilter<()>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let playground_filter = warp::path!("playground" / ..).map(move || {
        warp::reply::html(async_graphql::http::playground_source(
            GraphQLPlaygroundConfig::new("http://127.0.0.1:3030/api/v0/graphql"),
        ))
    });
    root.and(playground_filter).boxed()
}
