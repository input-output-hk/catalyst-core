use super::logic;
use crate::v0::endpoints::proposals::requests::ProposalsByVoteplanIdAndIndex;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn get_proposal(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_proposal(id, context).await))
}

pub async fn get_all_proposals(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_all_proposals(context).await))
}

pub async fn get_proposals_by_voteplan_id_and_index(
    body: ProposalsByVoteplanIdAndIndex,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        logic::get_proposals_by_voteplan_id_and_index(body, context).await,
    ))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::{
        migrations as db_testing,
        models::{
            challenges::{test as challenges_testing, Challenge},
            community_advisors_reviews::test as reviews_testing,
            proposals::{test as proposals_testing, *},
        },
    };
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use crate::v0::endpoints::proposals::requests::ProposalVoteplanIdAndIndexes;
    use warp::Filter;

    #[tokio::test]
    async fn get_proposal_by_id_handler() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let mut proposal: FullProposalInfo = proposals_testing::get_test_proposal();
        proposals_testing::populate_db_with_proposal(&proposal, pool);
        let challenge: Challenge =
            challenges_testing::get_test_challenge_with_fund_id(proposal.proposal.fund_id);
        challenges_testing::populate_db_with_challenge(&challenge, pool);

        let review = reviews_testing::get_test_advisor_review_with_proposal_id(
            proposal.proposal.proposal_id.parse().unwrap(),
        );
        reviews_testing::populate_db_with_advisor_review(&review, pool);
        proposal.proposal.reviews_count = 1;
        // build filter
        let filter = warp::path!(i32)
            .and(warp::get())
            .and(with_context)
            .and_then(get_proposal);

        let result = warp::test::request()
            .method("GET")
            .path("/1")
            .reply(&filter)
            .await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        println!("{}", String::from_utf8(result.body().to_vec()).unwrap());
        let result_proposal: FullProposalInfo =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(proposal, result_proposal);
    }

    #[tokio::test]
    async fn get_all_proposals_handler() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let proposal: FullProposalInfo = proposals_testing::get_test_proposal();
        proposals_testing::populate_db_with_proposal(&proposal, pool);
        let challenge: Challenge =
            challenges_testing::get_test_challenge_with_fund_id(proposal.proposal.fund_id);
        challenges_testing::populate_db_with_challenge(&challenge, pool);
        // build filter
        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_all_proposals);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_proposals: Vec<FullProposalInfo> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(vec![proposal], result_proposals);
    }

    #[tokio::test]
    async fn get_proposal_by_voteplan_id_and_index() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let proposal: FullProposalInfo = proposals_testing::get_test_proposal();
        proposals_testing::populate_db_with_proposal(&proposal, pool);
        let challenge: Challenge =
            challenges_testing::get_test_challenge_with_fund_id(proposal.proposal.fund_id);
        challenges_testing::populate_db_with_challenge(&challenge, pool);
        // build filter
        let filter = warp::any()
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(get_proposals_by_voteplan_id_and_index);

        let request = ProposalVoteplanIdAndIndexes {
            vote_plan_id: proposal.proposal.chain_voteplan_id.clone(),
            indexes: vec![proposal.proposal.chain_proposal_index],
        };

        let result = warp::test::request()
            .method("POST")
            .json(&vec![request])
            .reply(&filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_proposals: Vec<FullProposalInfo> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(vec![proposal], result_proposals);
    }
}
