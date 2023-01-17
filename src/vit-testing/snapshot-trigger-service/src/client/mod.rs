pub mod args;
pub mod rest;

use crate::client::rest::SnapshotRestClient;
use crate::client::Error::CannotConvertFromOutput;
use crate::config::JobParameters;
use crate::ContextState;
use jormungandr_lib::crypto::account::Identifier;
use jortestkit::prelude::WaitBuilder;
use num_traits::ToPrimitive;
use snapshot_lib::registration::{Delegations as VotingDelegations, VotingRegistration};
use std::fmt::Debug;
use tracing::{debug, instrument};
use voting_tools_rs::{SnapshotEntry, VotingPowerSource};

#[instrument(fields(
    snapshot_token = snapshot_token.clone().into(),
    snapshot_address = snapshot_address.clone().into(),
    )
)]
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

    debug!("Snapshot done: {snapshot_jobs_status:?}");
    let snapshot =
        snapshot_client.get_snapshot(snapshot_job_id, job_params.tag.unwrap_or_default())?;

    Ok(SnapshotResult {
        status: snapshot_jobs_status,
        snapshot: read_initials(snapshot)?,
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
        snapshot: read_initials(snapshot)?,
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
        snapshot: read_initials(snapshot)?,
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
            voting_registrations.push(VotingRegistration {
                stake_public_key: output.stake_key.to_string(),
                voting_power: output
                    .voting_power
                    .to_u64()
                    .ok_or_else(|| {
                        CannotConvertFromOutput("cannot extract voting power".to_string())
                    })?
                    .into(),
                reward_address: output.rewards_address.to_string(),
                delegations: match output.voting_power_source {
                    VotingPowerSource::Legacy(legacy) => VotingDelegations::Legacy(
                        Identifier::from_hex(&legacy.to_hex())
                            .map_err(|e| CannotConvertFromOutput(e.to_string()))?,
                    ),
                    VotingPowerSource::Delegated(delegated) => {
                        let mut new = vec![];
                        for (key, weight) in delegated {
                            new.push((
                                Identifier::from_hex(&key.to_hex())
                                    .map_err(|e| CannotConvertFromOutput(e.to_string()))?,
                                weight,
                            ));
                        }
                        VotingDelegations::New(new)
                    }
                },
                voting_purpose: *output.voting_purpose,
            });
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
    SerdeError(#[from] serde_json::Error),
    #[error("rest error")]
    ContextError(#[from] crate::context::Error),
    #[error("context error")]
    RestError(#[from] crate::client::rest::Error),
    #[error("rest error")]
    ChainError(#[from] chain_addr::Error),
    #[error(transparent)]
    Config(#[from] crate::config::Error),
    #[error("cannot convert voting registration from voting tools output due to: {0}")]
    CannotConvertFromOutput(String),
}
