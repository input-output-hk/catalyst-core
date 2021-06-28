use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use chain_addr::AddressReadable;
use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::InitialUTxO;
use jortestkit::prelude::WaitBuilder;
use snapshot_trigger_service::client::{
    do_snapshot as do_snapshot_internal, rest::SnapshotRestClient,
};
use snapshot_trigger_service::config::JobParameters;
use snapshot_trigger_service::State;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use vitup::setup::generate::read_initials;

pub fn do_snapshot(temp_dir: &TempDir, job_params: JobParameters) -> Result<SnapshotResult, Error> {
    let snapshot_token = std::env::var("SNAPSHOT_TOKEN").expect("SNAPSHOT_TOKEN not defined");
    let snapshot_address = std::env::var("SNAPSHOT_ADDRESS").expect("SNAPSHOT_ADDRESS not defined");

    do_snapshot_internal(temp_dir, job_params, snapshot_token, snapshot_address)
}

pub fn wait_for_db_sync() {
    println!("Waiting 5 mins before running snapshot");
    std::thread::sleep(std::time::Duration::from_secs(5 * 60));
    println!("Wait finished.");
}
