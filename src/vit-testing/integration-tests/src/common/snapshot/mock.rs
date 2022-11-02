use crate::common::snapshot::SnapshotServiceStarter;
use assert_fs::TempDir;
use mainnet_lib::InMemoryDbSync;
use snapshot_trigger_service::client::SnapshotResult;
use snapshot_trigger_service::config::{
    ConfigurationBuilder, JobParameters, NetworkType, VotingToolsParams,
};

pub fn do_snapshot(
    db_sync_instance: &InMemoryDbSync,
    job_parameters: JobParameters,
    testing_directory: &TempDir,
) -> SnapshotResult {
    let params = VotingToolsParams {
        bin: Some("snapshot_tool".to_string()),
        nix_branch: None,
        network: NetworkType::Mainnet,
        db: db_sync_instance
            .db_path()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string(),
        db_user: "fake".to_string(),
        db_pass: "fake".to_string(),
        db_host: "fake".to_string(),
        scale: 1_000_000,
        additional_params: vec![
            "dry_run".to_string(),
            "--mock-json-file".to_string(),
            db_sync_instance.db_path().to_str().unwrap().to_string(),
        ],
    };

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(params)
        .with_tmp_result_dir(testing_directory)
        .build();

    SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(testing_directory)
        .unwrap()
        .snapshot(job_parameters)
}
