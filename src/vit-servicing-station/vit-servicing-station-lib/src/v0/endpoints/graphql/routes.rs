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
        challenges::{test as challenges_testing, Challenge},
        funds::{test as funds_testing, Fund},
        proposals::{test as proposal_testing, FullProposalInfo},
    };

    use crate::db::migrations as db_testing;
    use crate::db::models::proposals::{ChallengeType, Proposal};
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
                    id,
                    challengeType,
                    title,
                    description,
                    rewardsTotal,
                    fundId,
                    challengeUrl
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
                id,
                challengeType,
                title,
                description,
                rewardsTotal,
                fundId,
                challengeUrl
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
                fundId,
                challengeId,
                challengeType
            }
        }"#;

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
            fundId,
            challengeId,
            challengeType
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
        FullProposalInfo,
        impl Filter<Extract = impl Reply, Error = Rejection> + Clone,
    ) {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let full_proposal: FullProposalInfo = proposal_testing::get_test_proposal();
        let proposal = &full_proposal.proposal;
        proposal_testing::populate_db_with_proposal(&full_proposal, &pool);
        let challenge: Challenge =
            challenges_testing::get_test_challenge_with_fund_id(proposal.fund_id);
        challenges_testing::populate_db_with_challenge(&challenge, &pool);

        // return filter
        (
            full_proposal,
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
                "id": proposal.proposal.proposal_id
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
        println!("{}", result_proposal);

        // due to the error in serde where flatten+alias do not work as expected,
        // https://github.com/serde-rs/serde/issues/1504
        // we need to manually deserialize the proposal parts
        let inner_proposal: Proposal = serde_json::from_value(result_proposal.clone()).unwrap();

        let challenge_type: ChallengeType =
            serde_json::from_value(result_proposal.get("challengeType").unwrap().clone()).unwrap();

        // this is not supported by GraphQL still, but is should be checked once it is implmented
        // let proposal_challenge_data: ProposalChallengeInfo =
        //     serde_json::from_value(result_proposal.clone()).unwrap();
        //
        // let result_proposal: FullProposalInfo = FullProposalInfo {
        //     proposal: inner_proposal,
        //     proposal_challenge_specific_data: proposal_challenge_data,
        //     challenge_type,
        // };

        assert_eq!(proposal.proposal, inner_proposal);
        assert_eq!(proposal.challenge_type, challenge_type);
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

        assert_eq!(vec![proposal.proposal], result_proposal);
    }
}
