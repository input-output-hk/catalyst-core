use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_fs::TempDir;

#[test]
pub fn service_version() {
    let temp_dir = TempDir::new().unwrap();
    let (hash, token) = data::token();

    let db_path = DbBuilder::new().with_token(token).build(&temp_dir).unwrap();
    let version = "TestV1".to_string();
    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path("non/existing/path")
        .with_service_version(version.clone())
        .start(&temp_dir)
        .unwrap();

    assert_eq!(
        server
            .rest_client_with_token(&hash)
            .service_version()
            .unwrap()
            .service_version,
        version
    );
}
