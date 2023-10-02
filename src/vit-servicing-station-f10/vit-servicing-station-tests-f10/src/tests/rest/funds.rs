use crate::common::{
    clients::RawRestClient,
    data,
    startup::{db::DbBuilder, quick_start, server::ServerBootstrapper},
};
use assert_fs::TempDir;
use reqwest::StatusCode;

#[test]
pub fn get_funds_list_is_not_empty() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir)?;
    server
        .rest_client_with_token(&snapshot.token_hash())
        .funds()
        .expect("cannot get funds");
    Ok(())
}

#[test]
pub fn get_funds_by_id() -> Result<(), Box<dyn std::error::Error>> {
    use pretty_assertions::assert_eq;
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let mut expected_fund = data::funds().first().unwrap().clone();
    let (hash, token) = data::token();

    let db_path = DbBuilder::new()
        .with_token(token)
        .with_funds(vec![expected_fund.clone()])
        .with_challenges(expected_fund.challenges.clone())
        .build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)?;

    let rest_client = server.rest_client_with_token(&hash);

    let actual_fund = rest_client.fund(&expected_fund.id.to_string())?;
    expected_fund.challenges.sort_by_key(|c| c.internal_id);
    assert_eq!(expected_fund, actual_fund);

    let rest_client: RawRestClient = server.rest_client_with_token(&hash).into();
    // non existing
    assert_eq!(rest_client.fund("2")?.status(), StatusCode::NOT_FOUND);
    // malformed index
    assert_eq!(rest_client.fund("a")?.status(), StatusCode::NOT_FOUND);
    // overflow index
    assert_eq!(
        rest_client.fund("3147483647")?.status(),
        StatusCode::NOT_FOUND
    );

    Ok(())
}
