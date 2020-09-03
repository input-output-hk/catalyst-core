use crate::common::{
    data,
    startup::{db::DbBuilder, quick_start, server::ServerBootstrapper},
};
use assert_fs::TempDir;
use reqwest::StatusCode;

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

#[test]
pub fn get_proposal_by_id() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let snapshot = data::Generator::new().snapshot();
    let expected_proposal = snapshot.proposals().first().unwrap().clone();
    let (hash, _token) = snapshot.any_token();
    let db_path = DbBuilder::new()
        .with_snapshot(&snapshot)
        .build(&temp_dir)
        .unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start()
        .unwrap();

    let rest_client = server.rest_client_with_token(&hash);

    let actual_proposal = rest_client
        .proposal(&expected_proposal.internal_id.to_string())
        .unwrap();
    assert_eq!(actual_proposal, expected_proposal);

    // non existing
    assert_eq!(
        rest_client.proposal_raw("2").unwrap().status(),
        StatusCode::NOT_FOUND
    );
    // malformed index
    assert_eq!(
        rest_client.proposal_raw("a").unwrap().status(),
        StatusCode::NOT_FOUND
    );
    // overflow index
    assert_eq!(
        rest_client.proposal_raw("3147483647").unwrap().status(),
        StatusCode::NOT_FOUND
    );
}
