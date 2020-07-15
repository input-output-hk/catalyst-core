use crate::common::startup::quick_start;
use assert_fs::TempDir;

#[test]
pub fn get_proposals_by_id() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal_id
        .parse()
        .unwrap();
    assert!(server
        .graphql_client_with_token(&snapshot.token_hash())
        .proposal_by_id(proposal_id)
        .is_ok());
}
