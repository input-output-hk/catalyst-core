use warp::{Filter, Rejection, Reply};

pub fn filter() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("health")
        .and(warp::get())
        .and_then(handler)
        .boxed()
}

pub async fn handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}
