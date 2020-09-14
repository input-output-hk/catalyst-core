use crate::common::startup::{empty_db, server::BootstrapCommandBuilder};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::{fixture::PathChild, TempDir};
use vit_servicing_station_lib::server::exit_codes::ApplicationExitCode;

#[test]
#[ignore = "https://github.com/input-output-hk/vit-servicing-station/issues/90"]
pub fn non_existing_block0_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .block0_path(temp_dir.child("block0.bin").path().to_str().unwrap())
        .db_url(empty_db(&temp_dir).to_str().unwrap())
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::DBConnectionError as i32);

    Ok(())
}

#[test]
#[ignore = "https://github.com/input-output-hk/vit-servicing-station/issues/90"]
pub fn malformed_path() {
    let temp_dir = TempDir::new().unwrap();
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .block0_path("C:/tmp/a:/block0.bin")
        .db_url(empty_db(&temp_dir).to_str().unwrap())
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::DBConnectionError as i32);
}

#[test]
#[ignore = "https://github.com/input-output-hk/vit-servicing-station/issues/90"]
pub fn network_path() {
    let temp_dir = TempDir::new().unwrap();
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .block0_path("//tmp/block0.bin")
        .db_url(empty_db(&temp_dir).to_str().unwrap())
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::DBConnectionError as i32);
}
