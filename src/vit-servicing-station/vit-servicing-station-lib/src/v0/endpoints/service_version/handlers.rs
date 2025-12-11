use super::logic;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn service_version(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::service_version(context).await))
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::v0::{
        context::test::new_test_shared_context_from_url,
        endpoints::service_version::schemas::ServiceVersion,
    };
    use vit_servicing_station_tests::common::startup::db::DbBuilder;
    use warp::Filter;

    #[tokio::test]
    async fn get_proposal_by_id_handler() {
        // build context
        let db_url = DbBuilder::new().build_async().await.unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // build filter
        let filter = warp::path::end()
            .and(warp::get())
            .and(with_context)
            .and_then(service_version);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        println!("{}", String::from_utf8(result.body().to_vec()).unwrap());
        let service_version_result: ServiceVersion =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();
        assert_eq!(service_version_result.service_version, "2.0".to_string());
    }
}
