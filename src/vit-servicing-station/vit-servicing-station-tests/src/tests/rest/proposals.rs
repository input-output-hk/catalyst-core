use crate::common::startup::quick_start;
use assert_fs::TempDir;

#[test]
pub fn get_proposals_list_is_not_empty() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();
    let proposals = server
        .rest_client_with_token(&snapshot.token_hash())
        .proposals()
        .expect("cannot get proposals");
    assert!(proposals.len() > 0);
}
