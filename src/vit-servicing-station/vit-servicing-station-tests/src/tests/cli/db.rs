use crate::common::cli::VitCliCommand;
use crate::common::startup::server::ServerBootstrapper;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::assert::PathAssert;
use assert_fs::{fixture::PathChild, TempDir};
use jortestkit::prelude::file_exists_and_not_empty;
#[test]
pub fn genereate_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_file = temp_dir.child("db.sqlite");
    let vit_cli: VitCliCommand = Default::default();
    vit_cli
        .db()
        .init()
        .db_url(db_file.path())
        .build()
        .assert()
        .success();

    db_file.assert(file_exists_and_not_empty());

    let server = ServerBootstrapper::new()
        .with_db_path(db_file.path().to_str().unwrap())
        .start()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));
    assert!(server.rest_client().health().is_ok());
}
