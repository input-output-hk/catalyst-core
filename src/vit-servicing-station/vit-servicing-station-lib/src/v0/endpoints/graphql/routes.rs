use super::schema::QueryRoot;
use crate::db;
use crate::v0::context::SharedContext;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
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
        |(schema, request): (
            async_graphql::Schema<QueryRoot, EmptyMutation, EmptySubscription>,
            async_graphql::Request,
        )| async move {
            // execute query
            let response = schema.execute(request).await;
            // return result
            Ok::<_, Infallible>(async_graphql_warp::Response::from(response))
        },
    );

    root.and(graph_ql).boxed()
}

#[cfg(test)]
mod test {
    use crate::db::models::{
        funds::{test as funds_testing, Fund},
        proposals::{test as proposal_testing, Proposal},
    };

    use crate::db::migrations as db_testing;
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use serde_json::json;
    use warp::{Filter, Rejection, Reply};

    const FUND_BY_ID_ALL_ATTRIBUTES_QUERY: &str = r#"
        query fundById($fid: Int!) {
            fund(id: $fid) {
                id,
                fundName,
                fundGoal,
                votingPowerInfo,
                votingPowerThreshold,
                rewardsInfo,
                fundStartTime,
                fundEndTime,
                nextFundStartTime,
                chainVotePlans {
                    id,
                    chainVoteplanId,
                    chainVoteStartTime,
                    chainVoteEndTime,
                    chainCommitteeEndTime,
                    chainVoteplanPayload,
                    chainVoteEncryptionKey,
                    fundId
                },
                challenges {
                    id
                    title
                    description
                    rewardsTotal
                    fundId    
                },
            }
        }"#;

    const FUNDS_ALL_ATTRIBUTES_QUERY: &str = r#"{
        funds {
            id,
            fundName,
            fundGoal,
            votingPowerInfo,
            votingPowerThreshold,
            rewardsInfo,
            fundStartTime,
            fundEndTime,
            nextFundStartTime,
            chainVotePlans {
                id,
                chainVoteplanId,
                chainVoteStartTime,
                chainVoteEndTime,
                chainCommitteeEndTime,
                chainVoteplanPayload,
                chainVoteEncryptionKey,
                fundId
            },
            challenges {
                id
                title
                description
                rewardsTotal
                fundId
            },
        }
    }"#;

    const PROPOSAL_BY_ID_ALL_ATTRIBUTES_QUERY: &str = r#"
         query proposalById($id: String!) {
            proposal(proposalId: $id) {
                internalId,
                proposalId,
                category {
                    categoryId,
                    categoryName,
                    categoryDescription,
                },
                proposalTitle,
                proposalSummary,
                proposalSolution,
                proposalProblem,
                proposalPublicKey,
                proposalFunds,
                proposalUrl,
                proposalFilesUrl,
                proposalImpactScore,
                proposer {
                    proposerName,
                    proposerEmail,
                    proposerUrl,
                    proposerRelevantExperience
                },
                chainProposalId,
                chainProposalIndex,
                chainVoteOptions,
                chainVoteplanId,
                chainVoteplanPayload,
                chainVoteEncryptionKey,
                chainVoteStartTime,
                chainVoteEndTime,
                chainCommitteeEndTime,
                fundId
                challengeId
            }
        }"#;

    // TODO: This query is not nice to read as documentation for the test. It was taken from the option
    // in postman to check the curl command. The actual graphql body request is like this:
    //     proposal(proposalId: 1) {
    //         id,
    //         proposalId,
    //         category {
    //             categoryId,
    //             categoryName,
    //             categoryDescription,
    //         },
    //         proposalTitle,
    //         proposalSummary,
    //         proposalProblem,
    //         proposalPublicKey,
    //         proposalFunds,
    //         proposalUrl,
    //         proposalFilesUrl,
    //         proposer {
    //             proposerName,
    //             proposerEmail,
    //             proposerUrl
    //         },
    //         chainProposalId,
    //         chainProposalIndex,
    //         chainVoteOptions,
    //         chainVoteplanId,
    //         chainVoteplanPayload,
    //         chainVoteEncryptionKey,
    //         chainVoteStartTime,
    //         chainVoteEndTime,
    //         chainCommitteeEndTime,
    //         fundId
    //     }
    // }
    // const PROPOSALS_ALL_ATTRIBUTES_QUERY: &str =  "{\"query\":\"{\\n    proposals {\\n        internalId,\\n        proposalId,\\n        category {\\n            categoryId,\\n            categoryName,\\n            categoryDescription,\\n        },\\n        proposalTitle,\\n        proposalSummary,\\n        proposalSolution,\\n        proposalProblem,\\n        proposalPublicKey,\\n        proposalFunds,\\n        proposalUrl,\\n        proposalFilesUrl,\\n        proposalImpactScore,\\n        proposer {\\n            proposerName,\\n            proposerEmail,\\n            proposerUrl\\n,        proposerRelevantExperience\\n        },\\n        chainProposalId,\\n        chainProposalIndex,\\n        chainVoteOptions,\\n        chainVoteplanId,\\n        chainVoteplanPayload,\\n        chainVoteEncryptionKey,\\n        chainVoteStartTime,\\n        chainVoteEndTime,\\n        chainCommitteeEndTime,\\n        fundId\\n    }\\n}\",\"variables\":{}}";
    const PROPOSALS_ALL_ATTRIBUTES_QUERY: &str = r#"{
        proposals {
            internalId,
            proposalId,
            category {
                categoryId,
                categoryName,
                categoryDescription,
            },
            proposalTitle,
            proposalSummary,
            proposalSolution,
            proposalProblem,
            proposalPublicKey,
            proposalFunds,
            proposalUrl,
            proposalFilesUrl,
            proposalImpactScore,
            proposer {
                proposerName,
                proposerEmail,
                proposerUrl,
                proposerRelevantExperience
            },
            chainProposalId,
            chainProposalIndex,
            chainVoteOptions,
            chainVoteplanId,
            chainVoteplanPayload,
            chainVoteEncryptionKey,
            chainVoteStartTime,
            chainVoteEndTime,
            chainCommitteeEndTime,
            fundId
            challengeId
        }
    }"#;

    async fn build_fund_test_filter() -> (
        Fund,
        impl Filter<Extract = impl Reply, Error = Rejection> + Clone,
    ) {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let fund: Fund = funds_testing::get_test_fund();
        funds_testing::populate_db_with_fund(&fund, &pool);

        // return filter
        (
            fund,
            super::filter(
                warp::any().and(warp::post()).boxed(),
                shared_context.clone(),
            )
            .await,
        )
    }

    async fn build_proposal_test_filter() -> (
        Proposal,
        impl Filter<Extract = impl Reply, Error = Rejection> + Clone,
    ) {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let proposal: Proposal = proposal_testing::get_test_proposal();
        proposal_testing::populate_db_with_proposal(&proposal, &pool);

        // return filter
        (
            proposal,
            super::filter(
                warp::any().and(warp::post()).boxed(),
                shared_context.clone(),
            )
            .await,
        )
    }

    #[tokio::test]
    async fn get_fund_by_id() {
        let (fund, graphql_filter) = build_fund_test_filter().await;
        let body = json!({
            "operationName": "fundById",
            "query": FUND_BY_ID_ALL_ATTRIBUTES_QUERY,
            "variables": {
                "fid": fund.id
            }
        })
        .to_string();

        let result = warp::test::request()
            .method("POST")
            .body(body)
            .reply(&graphql_filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);

        let query_result: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        if let Some(errors) = query_result.get("errors") {
            panic!("query had errors: {}", errors);
        }

        let result_fund = query_result["data"]["fund"].clone();
        let result_fund: Fund = serde_json::from_value(result_fund).unwrap();

        assert_eq!(fund, result_fund);
    }

    #[tokio::test]
    async fn get_funds() {
        let (fund, graphql_filter) = build_fund_test_filter().await;

        let body = json!({ "query": FUNDS_ALL_ATTRIBUTES_QUERY }).to_string();

        let result = warp::test::request()
            .method("POST")
            .body(body)
            .reply(&graphql_filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);

        let query_result: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        if let Some(errors) = query_result.get("errors") {
            panic!("query had errors: {}", errors);
        }

        let result_fund = query_result["data"]["funds"].clone();
        let result_fund: Vec<Fund> = serde_json::from_value(result_fund).unwrap();

        assert_eq!(vec![fund], result_fund);
    }

    #[tokio::test]
    async fn get_proposal_by_id() {
        let (proposal, graphql_filter) = build_proposal_test_filter().await;

        let body = json!({
            "operationName": "proposalById",
            "query": PROPOSAL_BY_ID_ALL_ATTRIBUTES_QUERY,
            "variables": {
                "id": proposal.proposal_id
            }
        })
        .to_string();

        let result = warp::test::request()
            .method("POST")
            .body(body)
            .reply(&graphql_filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);

        let query_result: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        if let Some(errors) = query_result.get("errors") {
            panic!("query had errors: {}", errors);
        }

        let result_proposal = query_result["data"]["proposal"].clone();
        let result_proposal: Proposal = serde_json::from_value(result_proposal).unwrap();

        assert_eq!(proposal, result_proposal);
    }

    #[tokio::test]
    async fn get_proposals() {
        let (proposal, graphql_filter) = build_proposal_test_filter().await;

        let body = json!({ "query": PROPOSALS_ALL_ATTRIBUTES_QUERY }).to_string();

        let result = warp::test::request()
            .method("POST")
            .body(body)
            .reply(&graphql_filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);

        let query_result: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        if let Some(errors) = query_result.get("errors") {
            panic!("query had errors: {}", errors);
        }

        let result_proposal = query_result["data"]["proposals"].clone();
        let result_proposal: Vec<Proposal> = serde_json::from_value(result_proposal).unwrap();

        assert_eq!(vec![proposal], result_proposal);
    }
}
