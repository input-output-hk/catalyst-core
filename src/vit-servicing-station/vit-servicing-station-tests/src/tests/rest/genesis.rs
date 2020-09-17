use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_fs::TempDir;

use crate::common::paths::BLOCK0_BIN;

#[test]
pub fn genesis_deserialize_bijection() {
    let temp_dir = TempDir::new().unwrap();
    let (hash, token) = data::token();

    let db_path = DbBuilder::new().with_token(token).build(&temp_dir).unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path(BLOCK0_BIN)
        .start()
        .unwrap();

    let expected = std::fs::read(BLOCK0_BIN).unwrap();

    let genesis_as_bytes = server
        .rest_client_with_token(&hash)
        .genesis()
        .expect("cannot get genesis block bytes");

    assert_eq!(expected, genesis_as_bytes);
}
