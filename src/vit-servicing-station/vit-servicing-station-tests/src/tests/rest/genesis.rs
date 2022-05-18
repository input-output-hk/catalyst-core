use crate::common::{
    clients::RawRestClient,
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_fs::TempDir;

use crate::common::paths::BLOCK0_BIN;
use hyper::StatusCode;

#[test]
pub fn genesis_deserialize_bijection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (hash, token) = data::token();

    let db_path = DbBuilder::new().with_token(token).build(&temp_dir).unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path(BLOCK0_BIN)
        .start(&temp_dir)
        .unwrap();

    let expected = std::fs::read(BLOCK0_BIN).unwrap();

    let genesis_as_bytes = server
        .rest_client_with_token(&hash)
        .genesis()
        .expect("cannot get genesis block bytes");

    assert_eq!(expected, genesis_as_bytes);
    Ok(())
}

#[test]
pub fn non_existing_block0() {
    let temp_dir = TempDir::new().unwrap();
    let (hash, token) = data::token();

    let db_path = DbBuilder::new().with_token(token).build(&temp_dir).unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path("non/existing/path")
        .start(&temp_dir)
        .unwrap();

    let rest_raw: RawRestClient = server.rest_client_with_token(&hash).into();

    assert_eq!(rest_raw.genesis().unwrap().status(), StatusCode::NO_CONTENT);
}
