pub mod args;
pub mod rest;

use crate::client::rest::SnapshotRestClient;
use crate::config::JobParameters;
use crate::State;
use jormungandr_lib::crypto::account::Identifier;
use jortestkit::prelude::WaitBuilder;
use thiserror::Error;
use voting_hir::VoterHIR;

pub fn do_snapshot<S: Into<String>, P: Into<String>>(
    job_params: JobParameters,
    snapshot_token: S,
    snapshot_address: P,
) -> Result<SnapshotResult, Error> {
    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());

    println!("Snapshot params: {:?}", job_params);
    let snapshot_job_id = snapshot_client.job_new(job_params.clone()).unwrap();
    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();

    println!("waiting for snapshot job");
    let snapshot_jobs_status =
        snapshot_client.wait_for_job_finish(snapshot_job_id.clone(), wait)?;

    println!("Snapshot done: {:?}", snapshot_jobs_status);
    let snapshot = snapshot_client.get_snapshot(
        snapshot_job_id,
        job_params.tag.unwrap_or_else(|| "".to_string()),
    )?;

    Ok(SnapshotResult {
        status: snapshot_jobs_status,
        snapshot: read_initials(&snapshot)?,
    })
}

pub fn get_snapshot_by_id<Q: Into<String>, S: Into<String>, P: Into<String>>(
    job_id: Q,
    tag: Q,
    snapshot_token: S,
    snapshot_address: P,
) -> Result<SnapshotResult, Error> {
    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());
    let job_id = job_id.into();

    let snapshot = snapshot_client.get_snapshot(job_id.clone(), tag.into())?;
    let status = snapshot_client.job_status(job_id)?;

    Ok(SnapshotResult {
        status: status?,
        snapshot: read_initials(&snapshot)?,
    })
}

pub fn get_snapshot_from_history_by_id<Q: Into<String>, S: Into<String>, P: Into<String>>(
    job_id: Q,
    tag: Q,
    snapshot_token: S,
    snapshot_address: P,
) -> Result<SnapshotResult, Error> {
    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());
    let job_id = job_id.into();

    let snapshot = snapshot_client.get_snapshot(job_id.clone(), tag.into())?;
    let status = snapshot_client.get_status(job_id)?;

    Ok(SnapshotResult {
        status,
        snapshot: read_initials(&snapshot)?,
    })
}

pub fn read_initials<S: Into<String>>(snapshot: S) -> Result<Vec<VoterHIR>, Error> {
    let snapshot = snapshot.into();
    serde_json::from_str(&snapshot).map_err(|_| Error::CannotParseSnapshotContent(snapshot.clone()))
}

#[derive(Debug)]
pub struct SnapshotResult {
    status: State,
    snapshot: Vec<VoterHIR>,
}

impl SnapshotResult {
    pub fn assert_status_is_finished(&self) {
        matches!(self.status, State::Finished { .. });
    }

    pub fn status(&self) -> State {
        self.status.clone()
    }

    pub fn initials(&self) -> &Vec<VoterHIR> {
        &self.snapshot
    }

    pub fn by_identifier(&self, identifier: &Identifier) -> Option<VoterHIR> {
        self.initials()
            .iter()
            .cloned()
            .find(|x| x.voting_key == *identifier)
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
    #[error(transparent)]
    Config(#[from] crate::config::Error),
}
