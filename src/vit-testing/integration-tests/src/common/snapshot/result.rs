#![allow(dead_code)]
#![allow(clippy::all)]
use jormungandr_lib::crypto::account::Identifier;
use jortestkit::prelude::WaitBuilder;
use mainnet_tools::snapshot::OutputExtension;
use snapshot_lib::registration::{Delegations as VotingDelegations, VotingRegistration};
use snapshot_trigger_service::client::rest::SnapshotRestClient;
use snapshot_trigger_service::config::JobParameters;
use snapshot_trigger_service::ContextState;
use std::fmt::Debug;
use voting_tools_rs::SnapshotEntry;

pub fn do_snapshot<S: Into<String> + Debug + Clone, P: Into<String> + Debug + Clone>(
    job_params: JobParameters,
    snapshot_token: S,
    snapshot_address: P,
) -> Result<SnapshotResult, Error> {
    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());

    let snapshot_job_id = snapshot_client.job_new(job_params.clone()).unwrap();
    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();

    let snapshot_jobs_status =
        snapshot_client.wait_for_job_finish(snapshot_job_id.clone(), wait)?;

    let snapshot =
        snapshot_client.get_snapshot(snapshot_job_id, job_params.tag.unwrap_or_default())?;

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
    let status = snapshot_client.get_status(job_id)?;

    Ok(SnapshotResult {
        status,
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

pub fn read_initials<S: Into<String>>(snapshot: S) -> Result<Vec<VotingRegistration>, Error> {
    let snapshot = snapshot.into();
    serde_json::from_str(&snapshot).map_err(Into::into)
}

#[derive(Debug)]
pub struct SnapshotResult {
    status: ContextState,
    snapshot: Vec<VotingRegistration>,
}

impl SnapshotResult {
    pub fn from_outputs(status: ContextState, snapshot: Vec<SnapshotEntry>) -> Result<Self, Error> {
        let mut voting_registrations = vec![];
        for output in snapshot {
            voting_registrations.push(output.try_into_voting_registration()?);
        }
        Ok(Self::new(status, voting_registrations))
    }

    pub fn new(status: ContextState, snapshot: Vec<VotingRegistration>) -> Self {
        Self { status, snapshot }
    }

    pub fn assert_status_is_finished(&self) {
        matches!(self.status, ContextState::Finished { .. });
    }

    pub fn status(&self) -> ContextState {
        self.status.clone()
    }

    pub fn registrations(&self) -> &Vec<VotingRegistration> {
        &self.snapshot
    }

    pub fn by_identifier(&self, identifier: &Identifier) -> Option<VotingRegistration> {
        self.registrations()
            .iter()
            .cloned()
            .find(|x| match &x.delegations {
                VotingDelegations::Legacy(id) => id == identifier,
                VotingDelegations::New(_dist) => false,
            })
    }

    pub fn by_delegation(&self, id: &Identifier) -> Result<Option<VotingRegistration>, Error> {
        Ok(self
            .registrations()
            .iter()
            .cloned()
            .find(|reg| match &reg.delegations {
                VotingDelegations::Legacy(delegation) => delegation == id,
                VotingDelegations::New(delegations) => {
                    delegations.iter().any(|(identifier, _)| identifier == id)
                }
            }))
    }

    pub fn contains_voting_key(&self, id: &Identifier) -> Result<bool, Error> {
        Ok(self.by_delegation(id)?.is_some())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("rest error")]
    Chain(#[from] chain_addr::Error),
    #[error(transparent)]
    Client(#[from] snapshot_trigger_service::client::rest::Error),
    #[error(transparent)]
    MainnetSnapshot(#[from] mainnet_tools::snapshot::Error),
}
