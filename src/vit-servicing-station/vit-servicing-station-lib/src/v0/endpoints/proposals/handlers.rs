use super::logic;
use crate::v0::endpoints::proposals::requests::ProposalsByVoteplanIdAndIndex;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn get_proposal(
    id: i32,
    voting_group: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        logic::get_proposal(id, voting_group, context).await,
    ))
}

pub async fn get_all_proposals(
    voting_group: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        logic::get_all_proposals(voting_group, context).await,
    ))
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

    use crate::v0::endpoints::proposals::requests::ProposalVoteplanIdAndIndexes;
    use crate::{
        db::models::proposals::FullProposalInfo,
        v0::context::test::new_test_shared_context_from_url,
    };
    use pretty_assertions::assert_eq;
    use vit_servicing_station_tests::common::data::ArbitrarySnapshotGenerator;
    use vit_servicing_station_tests::common::startup::db::DbBuilder;
    use warp::Filter;

    #[tokio::test]
    async fn get_proposal_by_id_handler() {
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

        let proposal = snapshot.proposals().into_iter().next().unwrap();

        // build filter
        let filter = warp::path!(i32 / String)
            .and(warp::get())
            .and(with_context)
            .and_then(get_proposal);

        let result = warp::test::request()
            .method("GET")
            .path(&format!(
                "/{}/{}",
                proposal.proposal.proposal_id, proposal.group_id
            ))
            .reply(&filter)
            .await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        println!("{}", String::from_utf8(result.body().to_vec()).unwrap());
        let result_proposal: FullProposalInfo =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(&proposal).unwrap(),
            serde_json::to_value(result_proposal).unwrap()
        );
    }

    #[tokio::test]
    async fn get_all_proposals_handler() {
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

        let proposals = snapshot.proposals();
        let first_proposal = proposals.into_iter().next().unwrap();
        let proposals: Vec<_> = snapshot
            .proposals()
            .into_iter()
            .filter(|p| p.group_id == first_proposal.group_id)
            .collect();

        // build filter
        let filter = warp::any()
            .and(warp::path!(String))
            .and(warp::get())
            .and(with_context)
            .and_then(get_all_proposals);

        let result = warp::test::request()
            .method("GET")
            .path(&format!("/{}", first_proposal.group_id))
            .reply(&filter)
            .await;
        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_proposals: Vec<FullProposalInfo> =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(proposals).unwrap(),
            serde_json::to_value(result_proposals).unwrap()
        );
    }

    #[tokio::test]
    async fn get_proposal_by_voteplan_id_and_index() {
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

        let proposal = snapshot.proposals().into_iter().next().unwrap();

        // build filter
        let filter = warp::any()
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(get_proposals_by_voteplan_id_and_index);

        let request = ProposalVoteplanIdAndIndexes {
            vote_plan_id: proposal.voteplan.chain_voteplan_id.clone(),
            indexes: vec![proposal.voteplan.chain_proposal_index],
        };

        let result = warp::test::request()
            .method("POST")
            .json(&vec![request])
            .reply(&filter)
            .await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let result_proposals: Vec<FullProposalInfo> =
            serde_json::from_str(core::str::from_utf8(result.body()).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(vec![proposal]).unwrap(),
            serde_json::to_value(result_proposals).unwrap()
        );
    }
}
