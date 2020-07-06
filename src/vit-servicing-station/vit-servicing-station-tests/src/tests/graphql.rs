use crate::common::startup::quick_start;
use assert_fs::TempDir;

#[test]
pub fn graphql_sanity_test() {
    let temp_dir = TempDir::new().unwrap();
    let (server, token) = quick_start(&temp_dir);
    assert!(server.graphql_with_token(token).proposal_by_id(1).is_ok());
}
