use snapshot_trigger_service::client::do_snapshot as do_snapshot_internal;
use snapshot_trigger_service::client::{Error, SnapshotResult};
use snapshot_trigger_service::config::JobParameters;

pub fn do_snapshot(job_params: JobParameters) -> Result<SnapshotResult, Error> {
    let snapshot_token = std::env::var("SNAPSHOT_TOKEN").expect("SNAPSHOT_TOKEN not defined");
    let snapshot_address = std::env::var("SNAPSHOT_ADDRESS").expect("SNAPSHOT_ADDRESS not defined");

    do_snapshot_internal(job_params, snapshot_token, snapshot_address)
}

pub fn wait_for_db_sync() {
    println!("Waiting 5 mins before running snapshot");
    std::thread::sleep(std::time::Duration::from_secs(5 * 60));
    println!("Wait finished.");
}
