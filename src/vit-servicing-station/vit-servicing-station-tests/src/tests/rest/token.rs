use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_fs::TempDir;
use reqwest::StatusCode;

#[test]
pub fn token_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (hash, token) = data::token();

    let db_path = DbBuilder::new().with_token(token).build(&temp_dir).unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_api_tokens(true)
        .start()
        .unwrap();

    let response = server.rest_client_with_token(&hash).health_raw()?;
    assert_eq!(response.status(), StatusCode::OK);

    let invalid_token = data::token_hash();
    let rest_client = server.rest_client_with_token(&invalid_token);
    assert_eq!(rest_client.health_raw()?.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        rest_client.fund_raw("1")?.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(rest_client.funds_raw()?.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        rest_client.proposal_raw("1")?.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        rest_client.proposals_raw()?.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        rest_client.genesis_raw()?.status(),
        StatusCode::UNAUTHORIZED
    );
    Ok(())
}
