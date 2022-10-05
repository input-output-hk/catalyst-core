use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn check_health(_context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use warp::Filter;

    #[tokio::test]
    async fn get_proposal_by_id_handler() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // build filter
        let filter = warp::path!("health" / ..)
            .and(warp::get())
            .and(with_context)
            .and_then(check_health);

        let result = warp::test::request()
            .method("GET")
            .path("/health")
            .reply(&filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);
    }
}
