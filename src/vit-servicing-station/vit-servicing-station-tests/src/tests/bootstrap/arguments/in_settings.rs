use crate::common::{
    data,
    paths::BLOCK0_BIN,
    startup::{
        db::DbBuilder,
        empty_db,
        server::{dump_settings, BootstrapCommandBuilder, ServerSettingsBuilder},
    },
};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::TempDir;
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use vit_servicing_station_lib::server::exit_codes::ApplicationExitCode;

#[test]
pub fn no_in_settings_provided() {
    let command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::DbConnectionError as i32);
}

#[test]
pub fn in_settings_file_does_not_exist() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();

    let non_existing_file = PathBuf::from_str("settings.json").unwrap();

    command_builder
        .in_settings_file(&non_existing_file)
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::LoadSettingsError as i32);
}

#[test]
pub fn in_settings_file_malformed() {
    let temp_dir = TempDir::new().unwrap();

    let mut settings_builder: ServerSettingsBuilder = Default::default();
    let settings = settings_builder
        .with_random_localhost_address()
        .with_db_path(empty_db(&temp_dir).to_str().unwrap())
        .with_block0_path(BLOCK0_BIN)
        .build();

    let settings_file = dump_settings(&temp_dir, &settings);
    remove_first_char_in_file(&settings_file);

    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .in_settings_file(&settings_file)
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::LoadSettingsError as i32);
}

pub fn remove_first_char_in_file(settings_file: &Path) {
    let mut file = File::open(settings_file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    drop(file);
    contents.remove(0);
    fs::write(settings_file, contents).expect("Unable to write file");
}

#[test]
pub fn in_settings_file_with_malformed_path() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();

    let non_existing_file = PathBuf::from_str("/tmp/a/c:/settings.json").unwrap();

    command_builder
        .in_settings_file(&non_existing_file)
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::LoadSettingsError as i32);
}

#[test]
pub fn db_url_and_block0_replaced() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();

    let (_, token) = data::token();

    let db_path = DbBuilder::new()
        .with_token(token)
        .with_proposals(data::proposals())
        .build(&temp_dir)?;

    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .block0_path(db_path.to_str().unwrap())
        .db_url(BLOCK0_BIN)
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::DbConnectionError as i32);
    Ok(())
}
