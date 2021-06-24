use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use chain_addr::AddressReadable;
use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::InitialUTxO;
use jortestkit::prelude::WaitBuilder;
use snapshot_trigger_service::client::rest::SnapshotRestClient;
use snapshot_trigger_service::config::JobParameters;
use snapshot_trigger_service::State;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use vitup::setup::generate::read_initials;

pub fn do_snapshot(temp_dir: &TempDir, job_params: JobParameters) -> Result<SnapshotResult, Error> {
    let snapshot_token = std::env::var("SNAPSHOT_TOKEN").expect("SNAPSHOT_TOKEN not defined");
    let snapshot_address = std::env::var("SNAPSHOT_ADDRESS").expect("SNAPSHOT_ADDRESS not defined");

    let snapshot_client = SnapshotRestClient::new_with_token(
        snapshot_token.to_string(),
        snapshot_address.to_string(),
    );

    println!("Snapshot params: {:?}", job_params);
    let snapshot_job_id = snapshot_client.job_new(job_params).unwrap();
    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();

    println!("waiting for snapshot job");
    let snapshot_jobs_status =
        snapshot_client.wait_for_job_finish(snapshot_job_id.clone(), wait)?;

    println!("Snapshot done: {:?}", snapshot_jobs_status);
    let snapshot_file = temp_dir.child("snapshot.json");
    snapshot_client.download_snapshot(snapshot_job_id, snapshot_file.path())?;

    Ok(SnapshotResult {
        status: snapshot_jobs_status,
        snapshot_file: snapshot_file.path().to_path_buf(),
    })
}

#[derive(Debug)]
pub struct SnapshotResult {
    status: State,
    snapshot_file: PathBuf,
}

impl SnapshotResult {
    pub fn assert_status_is_finished(&self) {
        matches!(self.status, State::Finished { .. });
    }

    pub fn status(&self) -> State {
        self.status.clone()
    }

    pub fn initials(&self) -> Result<Vec<Initial>, Error> {
        read_initials(&self.snapshot_file)
            .map_err(|_| Error::CannotParseSnapshotFile(self.snapshot_file.clone()))
    }

    pub fn by_address(&self, address: &str) -> Result<Option<InitialUTxO>, Error> {
        let address_readable = AddressReadable::from_str(address)?;
        for initial in self.initials()? {
            match initial {
                Initial::Fund(utxos) => {
                    return Ok(utxos.iter().cloned().find(|x| {
                        let address: chain_addr::Address = x.address.clone().into();
                        address_readable.to_address() == address
                    }));
                }
                _ => (),
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse file {0}")]
    CannotParseSnapshotFile(PathBuf),
    #[error("rest error")]
    RestError(#[from] snapshot_trigger_service::client::rest::Error),
    #[error("rest error")]
    ChainError(#[from] chain_addr::Error),
}

pub fn wait_for_db_sync() {
    println!("Waiting 5 mins before running snapshot");
    std::thread::sleep(std::time::Duration::from_secs(5 * 60));
    println!("Wait finished.");
}
