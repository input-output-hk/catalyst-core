use crate::common::startup::quick_start;
use assert_fs::TempDir;

#[test]
pub fn rest_sanity_test() {
    let temp_dir = TempDir::new().unwrap();
    let (server, hash) = quick_start(&temp_dir);
    let proposals = server
        .rest_with_token(hash)
        .proposals()
        .expect("cannot get proposals");
    assert!(proposals.len() > 0);
}
