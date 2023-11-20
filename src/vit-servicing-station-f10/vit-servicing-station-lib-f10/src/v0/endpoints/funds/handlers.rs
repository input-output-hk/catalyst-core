use super::logic;
use crate::v0::result::HandlerResult;
use crate::{db::models::funds::Fund, v0::context::SharedContext};
use warp::{Rejection, Reply};

pub async fn get_fund_by_id(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_fund_by_id(id, context).await))
}

pub async fn get_fund(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_current_fund(context).await))
}

pub async fn get_all_funds(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_all_funds(context).await))
}

pub async fn put_fund(fund: Fund, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::put_fund(fund, context).await))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::{
        migrations as db_testing,
        models::funds::{test as funds_testing, Fund},
        queries::funds::FundWithNext,
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
        let fund: Fund = funds_testing::get_test_fund(Some(1));
        let mut next_fund: Fund = funds_testing::get_test_fund(Some(2));

        next_fund.challenges = Vec::new();
        next_fund.chain_vote_plans = Vec::new();

        funds_testing::populate_db_with_fund(&fund, pool);
        funds_testing::populate_db_with_fund(&next_fund, pool);

        // build filter
        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_fund);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_fund: FundWithNext =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(fund, result_fund.fund);

        let next = result_fund.next.unwrap();
        assert_eq!(next_fund.id, next.id);
        assert_eq!(next_fund.fund_name, next.fund_name);
        assert_eq!(next_fund.stage_dates, next.stage_dates);
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
        let fund: Fund = funds_testing::get_test_fund(None);
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
    async fn get_all_funds_handler() {
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());

        let fund1: Fund = funds_testing::get_test_fund(Some(1));
        let mut fund2: Fund = funds_testing::get_test_fund(Some(2));

        fund2.challenges = vec![];
        fund2.chain_vote_plans = vec![];

        funds_testing::populate_db_with_fund(&fund1, pool);
        funds_testing::populate_db_with_fund(&fund2, pool);

        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_all_funds);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_funds: Vec<i32> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        assert_eq!(vec![fund1.id, fund2.id], result_funds);
    }

    #[tokio::test]
    async fn put_fund_handler() {
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());

        let fund1: Fund = funds_testing::get_test_fund(Some(1));
        let mut fund2: Fund = funds_testing::get_test_fund(Some(2));
        let mut fund3: Fund = funds_testing::get_test_fund(Some(3));

        fund2.challenges = vec![];
        fund2.chain_vote_plans = vec![];
        fund2.goals = vec![];

        fund3.challenges = vec![];
        fund3.chain_vote_plans = vec![];
        fund3.goals = vec![];

        funds_testing::populate_db_with_fund(&fund1, pool);
        funds_testing::populate_db_with_fund(&fund2, pool);

        let filter = warp::any()
            .and(warp::put())
            .and(warp::body::json())
            .and(with_context.clone())
            .and_then(put_fund);

        let mut updated_fund = fund2.clone();
        updated_fund.fund_name = "modified fund name".into();

        let result = warp::test::request()
            .method("PUT")
            .body(serde_json::to_string(&updated_fund).unwrap())
            .reply(&filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);

        let result_fund = test_get_fund(fund2.id, shared_context.clone()).await;
        assert_eq!(updated_fund, result_fund);

        let result_fund = test_get_fund(fund1.id, shared_context.clone()).await;
        assert_eq!(fund1, result_fund);

        assert_eq!(
            warp::test::request()
                .method("PUT")
                .body(serde_json::to_string(&fund3).unwrap())
                .reply(&filter)
                .await
                .status(),
            warp::http::StatusCode::OK
        );

        let result_fund = test_get_fund(fund3.id, shared_context.clone()).await;
        assert_eq!(fund3, result_fund);

        let result_fund = test_get_fund(fund2.id, shared_context.clone()).await;
        assert_eq!(updated_fund.clone(), result_fund.clone());
        // just to be extra sure
        assert_ne!(fund2, updated_fund);

        let result_fund = test_get_fund(fund1.id, shared_context.clone()).await;
        assert_eq!(fund1, result_fund);
    }

    async fn test_get_fund(id: i32, context: SharedContext) -> Fund {
        let with_context = warp::any().map(move || context.clone());

        let get_filter = warp::path!(i32)
            .and(warp::get())
            .and(with_context)
            .and_then(get_fund_by_id);

        let result = warp::test::request()
            .method("GET")
            .path(&format!("/{}", id))
            .reply(&get_filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);

        serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap()
    }
}
