use super::logic;
use crate::v0::context::SharedContext;
use crate::v0::result::HandlerResult;
use warp::{Rejection, Reply};

pub async fn get_fund_by_id(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_fund_by_id(id, context).await))
}

pub async fn get_fund(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_fund(context).await))
}

pub async fn search_fund_by_name(
    query: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        logic::search_fund_by_name(query, context).await,
    ))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::{
        migrations as db_testing,
        models::funds::{test as funds_testing, Fund},
    };
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use warp::Filter;

    #[tokio::test]
    async fn get_fund_handler() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let fund: Fund = funds_testing::get_test_fund();
        funds_testing::populate_db_with_fund(&fund, pool);

        // build filter
        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_fund);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_fund: Fund =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(fund, result_fund);
    }

    #[tokio::test]
    async fn get_fund_by_id_handler() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let fund: Fund = funds_testing::get_test_fund();
        funds_testing::populate_db_with_fund(&fund, pool);

        // build filter
        let filter = warp::path!(i32)
            .and(warp::get())
            .and(with_context)
            .and_then(get_fund_by_id);

        let result = warp::test::request()
            .method("GET")
            .path(&format!("/{}", fund.id))
            .reply(&filter)
            .await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_fund: Fund =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(fund, result_fund);
    }

    #[tokio::test]
    async fn search_fund_by_name_handler() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let fund: Fund = funds_testing::get_test_fund();
        funds_testing::populate_db_with_fund(&fund, pool);

        let fund_from_id_query: Fund = {
            let filter = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fund_by_id);

            let result = warp::test::request()
                .method("GET")
                .path(&format!("/{}", fund.id))
                .reply(&filter)
                .await;
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap()
        };

        // build filter
        let filter = warp::path!("search" / String)
            .and(warp::get())
            .and(with_context)
            .and_then(search_fund_by_name);

        let result = warp::test::request()
            .method("GET")
            .path("/search/hey")
            .reply(&filter)
            .await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result: Vec<Fund> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(fund, fund_from_id_query);
    }
}
