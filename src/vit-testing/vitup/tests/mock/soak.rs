use crate::mock::write_config;
use assert_cmd::cargo::CommandCargoExt;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use std::process::Command;
use std::process::Stdio;
use valgrind::ValgrindClient;
use vitup::client::rest::{VitupDisruptionRestClient, VitupRest};
use vitup::config;
use vitup::config::{Block0Initial, Block0Initials, Initials};
use vitup::mode::mock::Configuration;

#[test]
pub fn run_mock_with_restarts_for_15_minutes() {
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

    let mock_rest = VitupRest::new(format!("http://127.0.0.1:{}", configuration.port));

    assert!(mock_rest.is_up());

    let admin_mock_rest: VitupDisruptionRestClient = mock_rest.into();

    let client = ValgrindClient::new(
        format!("http://127.0.0.1:{}", configuration.port),
        Default::default(),
    )
    .unwrap();

    let reset_config = config::Config {
        initials: Initials {
            snapshot: None,
            block0: Block0Initials(vec![Block0Initial::AboveThreshold {
                above_threshold: 10,
                pin: "1234".to_string(),
                role: Default::default(),
            }]),
        },
        vote_plan: Default::default(),
        blockchain: Default::default(),
        data: Default::default(),
        service: Default::default(),
        additional: Default::default(),
    };

    for _ in 0..150 {
        client.funds().expect("failed to get funds from backend");
        client
            .challenges()
            .expect("failed to get challenges from backend");
        client
            .settings()
            .expect("failed to get settings from backend");

        let request = admin_mock_rest.reset_with_config(&reset_config);
        let request = request.unwrap();
        if request.status() != 200 {
            let request = admin_mock_rest.reset_with_config(&reset_config);
            assert_eq!(request.unwrap().status(), 200);
        }
        std::thread::sleep(std::time::Duration::from_secs(5))
    }
    mock_process.kill().unwrap();
}
