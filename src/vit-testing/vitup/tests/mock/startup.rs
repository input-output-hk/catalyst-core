use crate::mock::write_config;
use assert_cmd::cargo::CommandCargoExt;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use std::process::Command;
use std::process::Stdio;
use vitup::mode::mock::Configuration;

#[test]
pub fn start_mock() {
    let temp_dir = TempDir::new().unwrap();

    let configuration = Configuration {
        port: 10000,
        working_dir: temp_dir.child("mock").path().to_path_buf(),
        protocol: Default::default(),
        token: None,
        local: true,
    };

    let config_child = temp_dir.child("config.yaml");
    let config_file_path = config_child.path();
    write_config(&configuration, config_file_path);

    let mut cmd = Command::cargo_bin("vitup").unwrap();
    let mut mock_process = cmd
        .arg("start")
        .arg("mock")
        .arg("--config")
        .arg(config_file_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let request = reqwest::blocking::Client::new()
        .get(format!(
            "http://127.0.0.1:{}/api/health",
            configuration.port
        ))
        .send();

    assert_eq!(request.unwrap().status(), 200);

    mock_process.kill().unwrap();
}
