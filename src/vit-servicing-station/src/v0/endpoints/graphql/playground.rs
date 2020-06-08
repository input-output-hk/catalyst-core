use warp::{Filter, Rejection, Reply};

#[allow(dead_code)]
pub fn filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("playground" / ..)
        .map(move || {
            warp::reply::html(async_graphql::http::playground_source(
                "http://127.0.0.1:3030/api/v0/graphql",
                None,
            ))
        })
        .boxed()
}
