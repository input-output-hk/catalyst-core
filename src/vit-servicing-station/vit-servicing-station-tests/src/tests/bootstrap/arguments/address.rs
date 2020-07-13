use crate::common::startup::{quick_start, server::BootstrapCommandBuilder};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::TempDir;

#[test]
pub fn address_with_schema() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .address("http://127.0.0.1:8080")
        .build()
        .assert()
        .failure()
        .code(1);
}

#[test]
pub fn address_with_domain() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .address("http://localhost:8080")
        .build()
        .assert()
        .failure()
        .code(1);
}

#[test]
pub fn port_already_in_use() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, _) = quick_start(&temp_dir)?;

    let settings = server.settings();
    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .address(settings.address.to_string())
        .db_url(settings.db_url)
        .block0_path(settings.block0_path)
        .build()
        .assert()
        .failure()
        .code(101);
    Ok(())
}
