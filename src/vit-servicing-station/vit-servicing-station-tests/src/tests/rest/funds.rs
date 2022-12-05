use crate::common::{
    clients::RawRestClient,
    data,
    startup::{db::DbBuilder, quick_start, server::ServerBootstrapper},
};
use assert_fs::TempDir;
use reqwest::StatusCode;
use vit_servicing_station_lib::db::models::funds::Fund;

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
pub fn get_funds_by_id() {
    use pretty_assertions::assert_eq;
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let mut expected_fund = data::funds().first().unwrap().clone();
    normalize(&mut expected_fund);
    let (hash, token) = data::token();

    let db_path = DbBuilder::new()
        .with_token(token)
        .with_funds(vec![expected_fund.clone()])
        .with_challenges(expected_fund.challenges.clone())
        .with_groups(expected_fund.groups.iter().cloned().collect())
        .build(&temp_dir)
        .unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path)
        .start(&temp_dir)
        .unwrap();

    let rest_client = server.rest_client_with_token(&hash);

    let mut actual_fund = rest_client.fund(&expected_fund.id.to_string()).unwrap();

    normalize(&mut actual_fund);

    assert_eq!(expected_fund.id, actual_fund.id);

    let rest_client: RawRestClient = server.rest_client_with_token(&hash).into();
    // non existing
    assert_eq!(
        rest_client.fund("2").unwrap().status(),
        StatusCode::NOT_FOUND
    );
    // malformed index
    assert_eq!(
        rest_client.fund("a").unwrap().status(),
        StatusCode::NOT_FOUND
    );
    // overflow index
    assert_eq!(
        rest_client.fund("3147483647").unwrap().status(),
        StatusCode::NOT_FOUND
    );
}

fn normalize(fund: &mut Fund) {
    fund.challenges.sort_by_key(|fund| fund.id);
    fund.chain_vote_plans.sort_by_key(|c| c.id);
    fund.challenges.sort_by_key(|c| c.internal_id);
}
