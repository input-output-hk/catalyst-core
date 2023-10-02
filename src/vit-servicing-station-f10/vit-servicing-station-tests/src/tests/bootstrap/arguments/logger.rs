use crate::common::startup::server::BootstrapCommandBuilder;
use assert_cmd::assert::OutputAssertExt;

#[cfg(windows)]
use crate::common::{
    paths::BLOCK0_BIN,
    startup::{
        empty_db,
        server::{dump_settings, ServerSettingsBuilder},
    },
};
#[cfg(windows)]
use assert_fs::TempDir;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use vit_servicing_station_lib::server::exit_codes::ApplicationExitCode;

#[test]
pub fn wrong_log_level_provided() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .log_level("traceble")
        .build()
        .assert()
        .failure()
        .code(1);
}

#[test]
#[cfg(windows)]
pub fn malformed_logger_path_provided() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .log_file(&PathBuf::from("c:\\a\\d:\\logger"))
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::LoadSettingsError as i32);
}

#[test]
#[cfg(windows)]
pub fn in_settings_file_malformed_log_output_path() {
    let temp_dir = TempDir::new().unwrap();

    let mut settings_builder: ServerSettingsBuilder = Default::default();
    let settings = settings_builder
        .with_random_localhost_address()
        .with_db_path(empty_db(&temp_dir).to_str().unwrap())
        .with_block0_path(BLOCK0_BIN)
        .with_log_output_path(PathBuf::from("c:\\a\\d:\\logger"))
        .build();

    let settings_file = dump_settings(&temp_dir, &settings);
    let mut command_builder: BootstrapCommandBuilder = Default::default();

    command_builder
        .in_settings_file(&settings_file)
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::LoadSettingsError as i32);
}
