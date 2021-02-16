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
pub fn get_proposal_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let mut expected_proposal = data::proposals().first().unwrap().clone();
    let expected_challenge = data::challenges().first().unwrap().clone();
    expected_proposal.challenge_id = expected_challenge.id;
    // TODO: challenge_type should be retrieved from the view data
    // and checked against expected_challenge
    let (hash, token) = data::token();

    let db_path = DbBuilder::new()
        .with_token(token)
        .with_proposals(vec![expected_proposal.clone()])
        .with_challenges(vec![expected_challenge.clone()])
        .build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)
        .unwrap();

    let rest_client = server.rest_client_with_token(&hash);

    let actual_proposal = rest_client.proposal(&expected_proposal.internal_id.to_string())?;
    assert_eq!(actual_proposal, expected_proposal);

    // non existing
    assert_eq!(
        rest_client.proposal_raw("2")?.status(),
        StatusCode::NOT_FOUND
    );
    // malformed index
    assert_eq!(
        rest_client.proposal_raw("a")?.status(),
        StatusCode::NOT_FOUND
    );
    // overflow index
    assert_eq!(
        rest_client.proposal_raw("3147483647")?.status(),
        StatusCode::NOT_FOUND
    );

    Ok(())
}
