use super::logic;
use crate::v0::endpoints::votes::VoteCasterAndVoteplanId;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn get_vote_by_caster_and_voteplan_id(
    body: VoteCasterAndVoteplanId,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        logic::get_vote_by_caster_and_voteplan_id(body.caster, body.vote_plan_id, context).await,
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::models::vote::{test as votes_testing, *};
    use crate::v0::context::test::new_test_shared_context_from_url;
    use crate::v0::endpoints::votes::VoteCasterAndVoteplanId;
    use vit_servicing_station_tests::common::startup::db::DbBuilder;
    use warp::Filter;

    #[tokio::test]
    #[ignore] // This test needs to be in the new service.
    async fn get_vote_by_voteplan_id_and_caster() {
        // build context
        let db_url = DbBuilder::new().build_async().await.unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        let pool = &shared_context.read().await.db_connection_pool;
        let vote: Vote = votes_testing::get_test_vote();

        votes_testing::populate_db_with_vote(&vote, pool);

        // build filter
        let filter = warp::any()
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(get_vote_by_caster_and_voteplan_id);

        let request = VoteCasterAndVoteplanId {
            vote_plan_id: vote.voteplan_id.clone(),
            caster: vote.caster.clone(),
        };

        let result = warp::test::request()
            .method("POST")
            .json(&request)
            .reply(&filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_votes: Vec<Vote> =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();
        assert_eq!(vec![vote], result_votes);
    }
}
