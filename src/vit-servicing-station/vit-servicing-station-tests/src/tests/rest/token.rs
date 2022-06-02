use crate::common::{
    clients::RawRestClient,
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
        .start(&temp_dir)
        .unwrap();

    let invalid_token = data::token_hash();

    let rest_client: RawRestClient = server.rest_client_with_token(&hash).into();
    assert_eq!(rest_client.health()?.status(), StatusCode::OK);

    let rest_client: RawRestClient = server.rest_client_with_token(&invalid_token).into();
    assert_eq!(rest_client.health()?.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(rest_client.fund("1")?.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(rest_client.funds()?.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        rest_client.proposal("1")?.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(rest_client.proposals()?.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(rest_client.genesis()?.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
