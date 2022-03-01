use crate::common::data::multivoteplan_snapshot;
use crate::common::startup::{db::DbBuilder, server::ServerBootstrapper};
use assert_fs::TempDir;
use vit_servicing_station_lib::db::models::proposals::Proposal;
use vit_servicing_station_lib::v0::endpoints::proposals::ProposalVoteplanIdAndIndexes;
#[test]
pub fn get_proposals_by_voteplan_id_and_index() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let snapshot = multivoteplan_snapshot();

    let db_path = DbBuilder::new()
        .with_snapshot(&snapshot)
        .build(&temp_dir)
        .unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)
        .unwrap();

    let rest_client = server.rest_client_with_token(&snapshot.token_hash());
    let mut expected_proposals = rest_client.proposals().unwrap();

    let vote_plan_id: String = snapshot.voteplans()[0].chain_voteplan_id.clone();
    let indexes: Vec<i64> = expected_proposals
        .iter()
        .filter(|x| x.chain_voteplan_id == vote_plan_id)
        .map(|p| p.chain_proposal_index)
        .collect();

    expected_proposals = expected_proposals
        .into_iter()
        .filter(|x| x.chain_voteplan_id == vote_plan_id)
        .filter(|x| indexes.contains(&x.chain_proposal_index))
        .collect();

    let mut actual_proposals: Vec<Proposal> = rest_client
        .proposals_by_voteplan_id_and_index(&[ProposalVoteplanIdAndIndexes {
            vote_plan_id,
            indexes,
        }])
        .unwrap()
        .into_iter()
        .map(|proposal| proposal.proposal)
        .collect();

    expected_proposals.sort_by(|a, b| a.internal_id.cmp(&b.internal_id));
    actual_proposals.sort_by(|a, b| a.internal_id.cmp(&b.internal_id));
    assert_eq!(actual_proposals, expected_proposals);
}
