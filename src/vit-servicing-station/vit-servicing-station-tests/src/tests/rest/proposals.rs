use crate::common::{
    clients::RawRestClient,
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
    assert!(!proposals.is_empty());
}

#[test]
pub fn get_proposal_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let mut expected_proposal = data::proposals().first().unwrap().clone();
    let mut expected_challenge = data::challenges().first().unwrap().clone();
    expected_proposal.proposal.challenge_id = expected_challenge.id;
    expected_challenge.challenge_type = expected_proposal.challenge_type.clone();

    let (hash, token) = data::token();

    let db_path = DbBuilder::new()
        .with_token(token)
        .with_proposals(vec![expected_proposal.clone()])
        .with_challenges(vec![expected_challenge])
        .build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)
        .unwrap();

    let rest_client = server.rest_client_with_token(&hash);

    let actual_proposal =
        rest_client.proposal(&expected_proposal.proposal.internal_id.to_string())?;
    assert_eq!(actual_proposal, expected_proposal.proposal);
    let rest_client: RawRestClient = rest_client.into();
    // non existing
    assert_eq!(rest_client.proposal("2")?.status(), StatusCode::NOT_FOUND);
    // malformed index
    assert_eq!(rest_client.proposal("a")?.status(), StatusCode::NOT_FOUND);
    // overflow index
    assert_eq!(
        rest_client.proposal("3147483647")?.status(),
        StatusCode::NOT_FOUND
    );

    Ok(())
}
