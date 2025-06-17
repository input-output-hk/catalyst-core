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
    use crate::{
        db::{models::funds::Fund, queries::funds::FundWithNext},
        v0::context::test::new_test_shared_context_from_url,
    };
    use vit_servicing_station_tests::common::{
        data::ArbitrarySnapshotGenerator, startup::db::DbBuilder,
    };
    use warp::Filter;

    #[tokio::test]
    async fn get_fund_handler() {
        // build context
        let mut gen = ArbitrarySnapshotGenerator::default();
        let snapshot = gen.snapshot();

        let db_url = DbBuilder::new()
            .with_snapshot(&snapshot)
            .build_async()
            .await
            .unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        let mut funds = snapshot.funds().into_iter();
        let mut fund = funds.next().unwrap();
        fund.challenges
            .sort_by(|a, b| a.internal_id.cmp(&b.internal_id));
        let next_fund = funds.next().unwrap();

        // build filter
        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_fund);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let mut result_fund: FundWithNext =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();
        result_fund
            .fund
            .challenges
            .sort_by(|a, b| a.internal_id.cmp(&b.internal_id));
        assert_eq!(
            serde_json::to_value(fund).unwrap(),
            serde_json::to_value(result_fund.fund).unwrap()
        );

        let next = result_fund.next.unwrap();
        assert_eq!(next_fund.id, next.id);
        assert_eq!(next_fund.fund_name, next.fund_name);
        assert_eq!(
            serde_json::to_value(next_fund.stage_dates).unwrap(),
            serde_json::to_value(next.stage_dates).unwrap()
        );
    }

    #[tokio::test]
    async fn get_fund_by_id_handler() {
        // build context
        let mut gen = ArbitrarySnapshotGenerator::default();

        let snapshot = gen.snapshot();
        let funds = snapshot.funds();

        let db_url = DbBuilder::new()
            .with_snapshot(&snapshot)
            .build_async()
            .await
            .unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);

        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // build filter
        let filter = warp::path!(i32)
            .and(warp::get())
            .and(with_context)
            .and_then(get_fund_by_id);

        let result = warp::test::request()
            .method("GET")
            .path(&format!("/{}", funds[0].id))
            .reply(&filter)
            .await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let mut result_fund: Fund =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();
        result_fund
            .challenges
            .sort_by(|a, b| a.internal_id.cmp(&b.internal_id));

        // Even though vit_servicing_station_tests uses the Fund type from this crate,
        // this is not being recognized as such. So just do this to get the same type.
        let mut expected_fund: Fund =
            serde_json::from_value(serde_json::to_value(&funds[0]).unwrap()).unwrap();
        expected_fund
            .challenges
            .sort_by(|a, b| a.internal_id.cmp(&b.internal_id));

        assert_eq!(expected_fund, result_fund);
    }

    #[tokio::test]
    async fn get_all_funds_handler() {
        let mut gen = ArbitrarySnapshotGenerator::default();

        let snapshot = gen.snapshot();
        let funds_ids = snapshot
            .funds()
            .into_iter()
            .map(|f| f.id)
            .collect::<Vec<_>>();

        let db_url = DbBuilder::new()
            .with_snapshot(&snapshot)
            .build_async()
            .await
            .unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);

        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_all_funds);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_funds: Vec<i32> =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();

        assert_eq!(funds_ids, result_funds);
    }
}
