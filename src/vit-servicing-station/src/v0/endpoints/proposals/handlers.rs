use super::logic;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn get_proposal(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_proposal(id, context).await))
}

pub async fn get_all_proposals(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_all_proposals(context).await))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::{
        models::{proposals::*, vote_options::VoteOptions},
        schema::{proposals, voteplans},
        testing as db_testing, DBConnectionPool,
    };
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;

    use chrono::Utc;
    use diesel::{ExpressionMethods, RunQueryDsl};
    use warp::Filter;

    pub fn get_test_proposal() -> Proposal {
        Proposal {
            internal_id: 1,
            proposal_id: "foo_proposal".to_string(),
            proposal_category: Category {
                category_id: "".to_string(),
                category_name: "foo_category_name".to_string(),
                category_description: "".to_string(),
            },
            proposal_title: "the proposal".to_string(),
            proposal_summary: "the proposal summary".to_string(),
            proposal_problem: "the proposal problem".to_string(),
            proposal_solution: "the proposal solution".to_string(),
            proposal_public_key: "pubkey".to_string(),
            proposal_funds: 10000,
            proposal_url: "http://foo.bar".to_string(),
            proposal_files_url: "http://foo.bar/files".to_string(),
            proposer: Proposer {
                proposer_name: "tester".to_string(),
                proposer_email: "tester@tester.tester".to_string(),
                proposer_url: "http://tester.tester".to_string(),
            },
            chain_proposal_id: b"foobar".to_vec(),
            chain_proposal_index: 0,
            chain_vote_options: VoteOptions::parse_coma_separated_value("b,a,r"),
            chain_voteplan_id: "voteplain_id".to_string(),
            chain_vote_start_time: Utc::now().timestamp(),
            chain_vote_end_time: Utc::now().timestamp(),
            chain_committee_end_time: Utc::now().timestamp(),
            chain_voteplan_payload: "none".to_string(),
            fund_id: 1,
        }
    }

    pub fn populate_db_with_proposal(proposal: &Proposal, pool: &DBConnectionPool) {
        let connection = pool.get().unwrap();

        // insert the proposal information
        let values = (
            proposals::proposal_id.eq(proposal.proposal_id.clone()),
            proposals::proposal_category.eq(proposal.proposal_category.category_name.clone()),
            proposals::proposal_title.eq(proposal.proposal_title.clone()),
            proposals::proposal_summary.eq(proposal.proposal_summary.clone()),
            proposals::proposal_problem.eq(proposal.proposal_problem.clone()),
            proposals::proposal_solution.eq(proposal.proposal_solution.clone()),
            proposals::proposal_public_key.eq(proposal.proposal_public_key.clone()),
            proposals::proposal_funds.eq(proposal.proposal_funds.clone()),
            proposals::proposal_url.eq(proposal.proposal_url.clone()),
            proposals::proposal_files_url.eq(proposal.proposal_files_url.clone()),
            proposals::proposer_name.eq(proposal.proposer.proposer_name.clone()),
            proposals::proposer_contact.eq(proposal.proposer.proposer_email.clone()),
            proposals::proposer_url.eq(proposal.proposer.proposer_url.clone()),
            proposals::chain_proposal_id.eq(proposal.chain_proposal_id.clone()),
            proposals::chain_proposal_index.eq(proposal.chain_proposal_index),
            proposals::chain_vote_options.eq(proposal.chain_vote_options.as_csv_string()),
            proposals::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
        );
        diesel::insert_into(proposals::table)
            .values(values)
            .execute(&connection)
            .unwrap();

        // insert the related fund voteplan information
        let voteplan_values = (
            voteplans::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
            voteplans::chain_vote_start_time.eq(proposal.chain_vote_start_time),
            voteplans::chain_vote_end_time.eq(proposal.chain_vote_end_time),
            voteplans::chain_committee_end_time.eq(proposal.chain_committee_end_time),
            voteplans::chain_voteplan_payload.eq(proposal.chain_voteplan_payload.clone()),
            voteplans::fund_id.eq(proposal.fund_id),
        );

        diesel::insert_into(voteplans::table)
            .values(voteplan_values)
            .execute(&connection)
            .unwrap();
    }

    #[tokio::test]
    async fn get_proposal_by_id_handler() {
        // build context
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool);
        let proposal: Proposal = get_test_proposal();
        populate_db_with_proposal(&proposal, &pool);

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
        let result_proposal: Proposal =
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
        db_testing::initialize_db_with_migration(&pool);
        let proposal: Proposal = get_test_proposal();
        populate_db_with_proposal(&proposal, &pool);

        // build filter
        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_all_proposals);

        let result = warp::test::request().method("GET").reply(&filter).await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_proposals: Vec<Proposal> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();
        assert_eq!(vec![proposal], result_proposals);
    }
}
