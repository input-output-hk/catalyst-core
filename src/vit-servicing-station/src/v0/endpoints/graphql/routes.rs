use super::schema::QueryRoot;
use crate::db;
use crate::v0::context::SharedContext;
use async_graphql::{http::GQLResponse, EmptyMutation, EmptySubscription, QueryBuilder, Schema};
use std::convert::Infallible;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // load a connection pool for the graphql schema
    let db_connection_pool: db::DBConnectionPool =
        context.clone().read().await.db_connection_pool.clone();

    let schema = Schema::build(
        QueryRoot {
            db_connection_pool: db_connection_pool.clone(),
        },
        EmptyMutation,
        EmptySubscription,
    )
    .data(db_connection_pool)
    .finish();

    let graph_ql = async_graphql_warp::graphql(schema).and_then(
        |(schema, builder): (_, QueryBuilder)| async move {
            // Execute query
            let resp = builder.execute(&schema).await;
            // Return result
            Ok::<_, Infallible>(warp::reply::json(&GQLResponse(resp)).into_response())
        },
    );

    root.and(crate::v0::endpoints::graphql::playground::filter().or(graph_ql))
        .boxed()
}

#[cfg(test)]
mod test {
    use crate::db::models::{
        funds::{test as funds_testing, Fund},
        proposals::{test as proposal_testing, Proposal},
    };

    use crate::db::testing as db_testing;
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use warp::Filter;

    // FIXME: This query is not nice to read as documentation for the test. It was taken from the option
    // in postman to check the curl command. The actual graphql body request is like this:
    // {
    //     fund(id: 1) {
    //         id,
    //         fundName,
    //         fundGoal,
    //         votingPowerInfo,
    //         rewardsInfo,
    //         fundStartTime,
    //         fundEndTime,
    //         nextFundStartTime,
    //         chainVotePlans {
    //             id,
    //             chainVoteplanId,
    //             chainVoteStartTime,
    //             chainVoteEndTime,
    //             chainCommitteeEnd,
    //             chainVoteplanPayload,
    //             fundId
    //         },
    //     }
    // }
    const FUND_ALL_ATTRIBUTES_QUERY: &str = "{\"query\":\"{\\n    fund(id: 1) {\\n        id,\\n        fundName,\\n        fundGoal,\\n        votingPowerInfo,\\n        rewardsInfo,\\n        fundStartTime,\\n        fundEndTime,\\n        nextFundStartTime,\\n        chainVotePlans {\\n            id,\\n            chainVoteplanId,\\n            chainVoteStartTime,\\n            chainVoteEndTime,\\n            chainCommitteeEnd,\\n            chainVoteplanPayload,\\n            fundId\\n        },\\n    }\\n}\",\"variables\":{}}";

    #[tokio::test]
    async fn get_fund() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool);
        let fund: Fund = funds_testing::get_test_fund();
        funds_testing::populate_db_with_fund(&fund, &pool);

        // build filter

        let graphql_filter = super::filter(
            warp::any().and(warp::post()).boxed(),
            shared_context.clone(),
        )
        .await;

        let result = warp::test::request()
            .method("POST")
            .header("Content-Type", "application/graphql")
            .body(FUND_ALL_ATTRIBUTES_QUERY.as_bytes())
            .reply(&graphql_filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);

        let query_result: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        let result_fund = query_result["data"]["fund"].clone();
        let result_fund: Fund = serde_json::from_value(result_fund).unwrap();

        assert_eq!(fund, result_fund);
    }
}
