pub mod args;
pub mod rest;

use crate::client::rest::SnapshotRestClient;
use crate::config::JobParameters;
use crate::State;
use chain_addr::Address;
use chain_addr::AddressReadable;
use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::InitialUTxO;
use jortestkit::prelude::WaitBuilder;
use std::str::FromStr;
use thiserror::Error;

pub fn do_snapshot<S: Into<String>, P: Into<String>>(
    job_params: JobParameters,
    snapshot_token: S,
    snapshot_address: P,
) -> Result<SnapshotResult, Error> {
    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());

    println!("Snapshot params: {:?}", job_params);
    let snapshot_job_id = snapshot_client.job_new(job_params).unwrap();
    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();

    println!("waiting for snapshot job");
    let snapshot_jobs_status =
        snapshot_client.wait_for_job_finish(snapshot_job_id.clone(), wait)?;

    println!("Snapshot done: {:?}", snapshot_jobs_status);
    let snapshot = snapshot_client.get_snapshot(snapshot_job_id)?;

    Ok(SnapshotResult {
        status: snapshot_jobs_status,
        snapshot: read_initials(&snapshot)?,
    })
}

pub fn get_snapshot_by_id<Q: Into<String>, S: Into<String>, P: Into<String>>(
    job_id: Q,
    snapshot_token: S,
    snapshot_address: P,
) -> Result<SnapshotResult, Error> {
    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());
    let job_id = job_id.into();

    let snapshot = snapshot_client.get_snapshot(job_id.clone())?;
    let status = snapshot_client.job_status(job_id)?;

    Ok(SnapshotResult {
        status: status?,
        snapshot: read_initials(&snapshot)?,
    })
}

pub fn get_snapshot_from_history_by_id<Q: Into<String>, S: Into<String>, P: Into<String>>(
    job_id: Q,
    snapshot_token: S,
    snapshot_address: P,
) -> Result<SnapshotResult, Error> {
    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());
    let job_id = job_id.into();

    let snapshot = snapshot_client.get_snapshot(job_id.clone())?;
    let status = snapshot_client.get_status(job_id)?;

    Ok(SnapshotResult {
        status,
        snapshot: read_initials(&snapshot)?,
    })
}

pub fn read_initials<S: Into<String>>(snapshot: S) -> Result<Vec<Initial>, Error> {
    let snapshot = snapshot.into();
    let value: serde_json::Value = serde_json::from_str(&snapshot)?;
    let initial = serde_json::to_string(&value["initial"])?;
    serde_json::from_str(&initial).map_err(|_| Error::CannotParseSnapshotContent(snapshot.clone()))
}

#[derive(Debug)]
pub struct SnapshotResult {
    status: State,
    snapshot: Vec<Initial>,
}

impl SnapshotResult {
    pub fn assert_status_is_finished(&self) {
        matches!(self.status, State::Finished { .. });
    }

    pub fn status(&self) -> State {
        self.status
    }

    pub fn initials(&self) -> &Vec<Initial> {
        &self.snapshot
    }

    pub fn by_address_str(&self, address: &str) -> Result<Option<InitialUTxO>, Error> {
        let address_readable = AddressReadable::from_str(address)?;
        for initial in self.initials() {
            if let Initial::Fund(utxos) = initial {
                return Ok(utxos.iter().cloned().find(|x| {
                    let address: chain_addr::Address = x.address.clone().into();
                    address_readable.to_address() == address
                }));
            }
        }
        Ok(None)
    }

    pub fn by_address(&self, expected_address: &Address) -> Result<Option<InitialUTxO>, Error> {
        for initial in self.initials() {
            if let Initial::Fund(utxos) = initial {
                return Ok(utxos.iter().cloned().find(|x| {
                    let address: chain_addr::Address = x.address.clone().into();
                    *expected_address == address
                }));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse file {0}")]
    CannotParseSnapshotContent(String),
    #[error("rest error")]
    ContextError(#[from] crate::context::Error),
    #[error("context error")]
    RestError(#[from] crate::client::rest::Error),
    #[error("rest error")]
    ChainError(#[from] chain_addr::Error),
    #[error("serialization error")]
    SerdeError(#[from] serde_json::Error),
}
