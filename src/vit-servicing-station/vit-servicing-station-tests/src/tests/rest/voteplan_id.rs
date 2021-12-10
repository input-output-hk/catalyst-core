use crate::common::data::Snapshot;
use crate::common::startup::{db::DbBuilder, server::ServerBootstrapper};
use assert_fs::TempDir;
use quickcheck::Arbitrary;
use quickcheck::QuickCheck;
use vit_servicing_station_lib::v0::endpoints::proposals::ProposalVoteplanIdAndIndexes;
#[test]
pub fn get_proposals_by_voteplan_id_and_index() {
    fn quick_test(snapshot: Snapshot) {
        let temp_dir = TempDir::new().unwrap().into_persistent();

        let db_path = DbBuilder::new()
            .with_snapshot(&snapshot)
            .build(&temp_dir)
            .unwrap();

        let server = ServerBootstrapper::new()
            .with_db_path(db_path.to_str().unwrap())
            .start(&temp_dir)
            .unwrap();

        let rest_client = server.rest_client_with_token(&snapshot.token_hash());
        let expected_proposals = rest_client.proposals().unwrap();

        let vote_plan_id: String = snapshot.voteplans()[0].chain_voteplan_id.clone();
        let indexes: Vec<i64> = expected_proposals
            .iter()
            .filter(|x| x.chain_voteplan_id == vote_plan_id)
            .map(|p| p.chain_proposal_index)
            .collect();

        let actual_proposals = rest_client
            .proposals_by_voteplan_id_and_index(&vec![ProposalVoteplanIdAndIndexes {
                vote_plan_id,
                indexes: indexes.clone(),
            }])
            .unwrap();

        assert!(actual_proposals.len() > 0);

        for proposal in actual_proposals {
            let expected_proposal = expected_proposals
                .iter()
                .find(|x| proposal.proposal.internal_id == x.internal_id)
                .unwrap();

            assert_eq!(&proposal.proposal, expected_proposal);
        }
    }

    QuickCheck::new()
        .max_tests(1)
        .quicktest(quick_test as fn(Snapshot))
        .unwrap();
}
