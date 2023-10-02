use crate::common::startup::server::BootstrapCommandBuilder;
use assert_cmd::assert::OutputAssertExt;
use vit_servicing_station_lib::server::exit_codes::ApplicationExitCode;

#[test]
pub fn malformed_path() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .db_url("C:/tmp/a:/databse.db")
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::DbConnectionError as i32);
}

#[test]
pub fn path_doesnt_exist() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .db_url("C:/foo.db")
        .build()
        .assert()
        .failure()
        .code(ApplicationExitCode::DbConnectionError as i32);
}
