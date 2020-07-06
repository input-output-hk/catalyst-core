use crate::common::startup::quick_start;
use assert_fs::TempDir;

#[test]
pub fn get_proposals_list_is_not_empty() {
    let temp_dir = TempDir::new().unwrap();
    let (server, hash) = quick_start(&temp_dir);
    let proposals = server
        .rest_client_with_token(hash)
        .proposals()
        .expect("cannot get proposals");
    assert!(proposals.len() > 0);
}
