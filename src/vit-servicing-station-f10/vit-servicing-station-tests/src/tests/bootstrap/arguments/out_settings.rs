use crate::common::{
    paths::BLOCK0_BIN,
    startup::{
        empty_db,
        server::{dump_settings, load_settings, BootstrapCommandBuilder, ServerSettingsBuilder},
    },
};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::{fixture::PathChild, TempDir};
use std::path::PathBuf;
use vit_servicing_station_lib::server::settings::LogLevel;
use vit_servicing_station_lib::server::settings::ServiceSettings;

#[test]
pub fn out_settings_provided() {
    let temp_dir = TempDir::new().unwrap();

    let (in_settings_file, settings) = example_settings_file(&temp_dir);
    let out_settings_file = temp_dir.child("out_settings.json");

    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .in_settings_file(&in_settings_file)
        .out_settings_file(out_settings_file.path())
        .build()
        .assert()
        .success();

    let actual_settings = load_settings(&out_settings_file.path());
    assert_eq!(settings, actual_settings);
}

#[test]
pub fn out_settings_file_override() {
    let temp_dir = TempDir::new().unwrap();
    let mut command_builder: BootstrapCommandBuilder = Default::default();

    let (in_settings_file, _) = example_settings_file(&temp_dir);

    command_builder
        .in_settings_file(&in_settings_file)
        .out_settings_file(&in_settings_file)
        .build()
        .assert()
        .success();
}

#[test]
#[ignore = "issue in comment for https://github.com/input-output-hk/vit-servicing-station/pull/75"]
pub fn out_settings_file_from_cmdline() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let (_, settings) = example_settings_file(&temp_dir);
    let out_settings_file = temp_dir.child("settings.json");

    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .db_url(&settings.db_url)
        .block0_path(&settings.block0_path)
        .address(settings.address.to_string())
        .out_settings_file(out_settings_file.path())
        .build()
        .assert()
        .success();

    let actual_settings = load_settings(&out_settings_file.path());
    assert_eq!(settings, actual_settings);
}

fn example_settings_file(temp_dir: &TempDir) -> (PathBuf, ServiceSettings) {
    let mut settings_builder: ServerSettingsBuilder = Default::default();
    let settings = settings_builder
        .with_random_localhost_address()
        .with_db_path(empty_db(temp_dir).to_str().unwrap())
        .with_block0_path(BLOCK0_BIN)
        .with_log_level(LogLevel::Info)
        .with_log_output_path(temp_dir.child("logger.log").path().into())
        .build();
    let settings_file = dump_settings(temp_dir, &settings);
    (settings_file, settings)
}
