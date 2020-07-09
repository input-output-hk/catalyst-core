use crate::common::startup::quick_start;
use assert_fs::TempDir;

#[test]
pub fn get_proposals_by_id() {
    let temp_dir = TempDir::new().unwrap();
    let (server, token) = quick_start(&temp_dir).unwrap();
    assert!(server
        .graphql_client_with_token(&token)
        .proposal_by_id(1)
        .is_ok());
}
