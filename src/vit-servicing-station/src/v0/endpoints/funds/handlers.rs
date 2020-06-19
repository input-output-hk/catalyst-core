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

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::{models::funds::Fund, schema::funds, testing as db_testing, DBConnectionPool};
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;

    use chrono::Utc;
    use diesel::{ExpressionMethods, RunQueryDsl};
    use warp::Filter;

    pub fn get_test_fund() -> Fund {
        Fund {
            id: 1,
            fund_name: "hey oh let's go".to_string(),
            fund_goal: "test this endpoint".to_string(),
            voting_power_info: ">9000".to_string(),
            rewards_info: "not much".to_string(),
            fund_start_time: Utc::now(),
            fund_end_time: Utc::now(),
            next_fund_start_time: Utc::now(),
            chain_vote_plans: vec![],
        }
    }

    pub fn populate_db_with_fund(fund: &Fund, pool: &DBConnectionPool) {
        let connection = pool.get().unwrap();
        let values = (
            funds::fund_name.eq(fund.fund_name.clone()),
            funds::fund_goal.eq(fund.fund_goal.clone()),
            funds::voting_power_info.eq(fund.voting_power_info.clone()),
            funds::rewards_info.eq(fund.rewards_info.clone()),
            funds::fund_start_time.eq(fund.fund_start_time.timestamp()),
            funds::fund_end_time.eq(fund.fund_end_time.timestamp()),
            funds::next_fund_start_time.eq(fund.next_fund_start_time.timestamp()),
        );
        diesel::insert_into(funds::table)
            .values(values)
            .execute(&connection)
            .unwrap();
    }

    #[tokio::test]
    async fn fund_by_id() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool);
        let fund: Fund = get_test_fund();
        populate_db_with_fund(&fund, &pool);

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
}
