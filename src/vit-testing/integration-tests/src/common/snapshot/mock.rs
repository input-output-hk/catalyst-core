use crate::common::snapshot::SnapshotServiceStarter;
use assert_fs::TempDir;
use mainnet_tools::db_sync::DbSyncInstance;
use mainnet_tools::voting_tools::VotingToolsMock;
use snapshot_trigger_service::client::SnapshotResult;
use snapshot_trigger_service::config::{ConfigurationBuilder, JobParameters};

pub fn do_snapshot(
    db_sync_instance: &DbSyncInstance,
    job_parameters: JobParameters,
    testing_directory: &TempDir,
) -> SnapshotResult {
    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap()
        .snapshot(job_parameters)
}
