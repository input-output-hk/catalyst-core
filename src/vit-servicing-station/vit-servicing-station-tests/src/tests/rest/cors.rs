use crate::common::{
    clients::RawRestClient,
    data::ArbitrarySnapshotGenerator,
    startup::{
        db::DbBuilder,
        server::{BootstrapCommandBuilder, ServerBootstrapper},
    },
};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::TempDir;

#[test]
pub fn cors_illegal_domain() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_allowed_origins("http://domain.com")
        .start(&temp_dir)?;

    let mut rest_client = server.rest_client_with_token(&snapshot.token_hash());
    rest_client.set_origin("http://other_domain.com");

    assert_request_failed_due_to_cors(&rest_client.into())?;
    Ok(())
}

fn assert_request_failed_due_to_cors(
    rest_client: &RawRestClient,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        rest_client.funds()?.text()?,
        "CORS request forbidden: origin not allowed"
    );
    Ok(())
}

#[test]
pub fn cors_malformed_domain_no_http() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .db_url(db_path.to_str().unwrap())
        .allowed_origins("domain.com")
        .build()
        .assert()
        .failure();
    Ok(())
}

#[test]
pub fn cors_ip_versus_domain() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_allowed_origins("http://127.0.0.1")
        .start(&temp_dir)?;

    let mut rest_client = server.rest_client_with_token(&snapshot.token_hash());
    rest_client.set_origin("http://localhost");

    assert_request_failed_due_to_cors(&rest_client.into())?;

    Ok(())
}

#[test]
pub fn cors_wrong_delimiter() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .db_url(db_path.to_str().unwrap())
        .allowed_origins("http://domain.com,http://other_domain.com")
        .build()
        .assert()
        .failure();
    Ok(())
}

#[test]
pub fn cors_single_domain() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_allowed_origins("http://domain.com")
        .start(&temp_dir)?;

    let mut rest_client: RawRestClient =
        server.rest_client_with_token(&snapshot.token_hash()).into();
    rest_client.set_origin("http://domain.com");

    assert!(rest_client.funds()?.status().is_success());

    Ok(())
}

#[test]
pub fn cors_https() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_allowed_origins("https://domain.com")
        .start(&temp_dir)?;

    let mut rest_client: RawRestClient =
        server.rest_client_with_token(&snapshot.token_hash()).into();
    rest_client.set_origin("https://domain.com");

    assert!(rest_client.funds()?.status().is_success());

    Ok(())
}

#[test]
pub fn cors_multi_domain() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_allowed_origins("http://domain.com;http://other_domain.com")
        .start(&temp_dir)?;

    let mut rest_client: RawRestClient =
        server.rest_client_with_token(&snapshot.token_hash()).into();
    rest_client.set_origin("http://other_domain.com");
    assert!(rest_client.funds()?.status().is_success());

    rest_client.set_origin("http://domain.com");
    assert!(rest_client.funds()?.status().is_success());

    assert!(!server.logger().any_error());

    Ok(())
}
