use crate::common::get_available_port;
use crate::common::snapshot::{result::SnapshotResult, SnapshotServiceStarter};
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use mainnet_lib::{DbSyncError, InMemoryDbSync};
use snapshot_trigger_service::config::{
    ConfigurationBuilder, JobParameters, NetworkType, VotingToolsParams,
};

pub fn do_snapshot(
    db_sync_instance: &InMemoryDbSync,
    job_parameters: JobParameters,
    testing_directory: &TempDir,
) -> Result<SnapshotResult, Error> {
    let mock_json_file = testing_directory.child("database.json");
    db_sync_instance.persist(mock_json_file.path())?;

    println!("do_snapshot::MemoryDB: {:#?}", db_sync_instance);

    let params = VotingToolsParams {
        bin: Some("snapshot_tool".to_string()),
        nix_branch: None,
        network: NetworkType::Mainnet,
        db: "".to_string(), //fake
        db_user: "".to_string(),
        db_pass: "".to_string(),
        db_host: "".to_string(),
        additional_params: Some(vec![
            "dry-run".to_string(),
            "--mock-json-file".to_string(),
            mock_json_file.path().to_str().unwrap().to_string(),
        ]),
    };

    let configuration = ConfigurationBuilder::default()
        .with_port(get_available_port())
        .with_voting_tools_params(params)
        .with_tmp_result_dir(testing_directory)
        .build();

    Ok(SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(testing_directory)?
        .snapshot(job_parameters)?)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DbSync(#[from] DbSyncError),
    #[error(transparent)]
    SnapshotIntegration(#[from] crate::common::snapshot::Error),
    #[error(transparent)]
    Snapshot(#[from] crate::common::snapshot::result::Error),
}
