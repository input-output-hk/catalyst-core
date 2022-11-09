pub mod args;
pub mod rest;

use crate::{client::rest::RegistrationRestClient, context::State, request::Request};
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use jormungandr_automation::jcli::JCli;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use jortestkit::prelude::WaitBuilder;
use math::round;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub fn do_registration(
    request: Request,
    registration_client: &RegistrationRestClient,
    temp_dir: &TempDir,
) -> RegistrationResult {
    let registration_job_id = registration_client.job_new(request.clone()).unwrap();

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();
    println!("waiting for registration job");
    let registration_jobs_status = registration_client
        .wait_for_job_finish(registration_job_id.clone(), wait)
        .unwrap();
    println!("{:?}", registration_jobs_status);

    let qr_code_path = temp_dir.child("qr_code");
    std::fs::create_dir_all(qr_code_path.path()).unwrap();

    if request.is_legacy() {
        let qr_code = registration_client
            .download_qr(registration_job_id.clone(), qr_code_path.path())
            .unwrap();
        let voting_sk = registration_client
            .get_catalyst_sk(registration_job_id)
            .unwrap();
        RegistrationResult::Legacy(LegacyResultInfo {
            qr_code,
            voting_sk,
            status: registration_jobs_status,
        })
    } else {
        RegistrationResult::Delegation(DelegationResultInfo {
            delegations: request.delegations(),
            status: registration_jobs_status,
        })
    }
}

#[derive(Debug)]
pub enum RegistrationResult {
    Legacy(LegacyResultInfo),
    Delegation(DelegationResultInfo),
}

#[derive(Clone, Debug)]
pub struct DelegationResultInfo {
    pub delegations: HashMap<String, u32>,
    pub status: State,
}

#[derive(Clone, Debug)]
pub struct LegacyResultInfo {
    pub qr_code: PathBuf,
    voting_sk: String,
    status: State,
}

impl LegacyResultInfo {
    pub fn identifier_as_str(&self) -> String {
        let jcli = JCli::new(PathBuf::from_str("jcli").expect("jcli not found on env"));
        jcli.key().convert_to_public_string(&self.voting_sk)
    }

    pub fn identifier(&self) -> Result<Identifier, Error> {
        Ok(Identifier::from_str(&self.identifier_as_str())?)
    }

    pub fn snapshot_entry(&self) -> Result<(Identifier, Value), Error> {
        Ok((self.identifier()?, self.funds_in_ada()?.into()))
    }

    pub fn status(&self) -> State {
        self.status.clone()
    }

    pub fn slot_no(&self) -> Option<u64> {
        match self.status() {
            State::Finished {
                info: Some(info), ..
            } => Some(info.slot_no),
            _ => None,
        }
    }

    pub fn print_snapshot_entry(&self) -> Result<(), Error> {
        println!(
            "[identifier: {}, funds:{}",
            self.identifier_as_str(),
            self.funds_in_ada()?
        );
        Ok(())
    }

    pub fn leak_sk(&self) -> String {
        self.voting_sk.clone()
    }

    pub fn funds_in_ada(&self) -> Result<u64, Error> {
        let funds = self.funds_in_lovelace()?;
        let rounded = round::floor(funds as f64, -6);
        Ok((rounded as u64) / 1_000_000)
    }

    pub fn funds_in_lovelace(&self) -> Result<u64, Error> {
        match &self.status {
            State::Finished { info, .. } => Ok(info
                .as_ref()
                .ok_or(Error::CannotGetFundsFromRegistrationResult)?
                .funds),
            _ => Err(Error::CannotGetFundsFromRegistrationResult),
        }
    }
}

impl RegistrationResult {
    pub fn as_legacy_registration(&self) -> Option<LegacyResultInfo> {
        match &self {
            Self::Legacy(info) => Some(info.clone()),
            Self::Delegation { .. } => None,
        }
    }

    pub fn as_delegation_registration(&self) -> Option<DelegationResultInfo> {
        match &self {
            Self::Legacy(_info) => None,
            Self::Delegation(info) => Some(info.clone()),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot get funds from registration result")]
    CannotGetFundsFromRegistrationResult,
    #[error("cannot get address from registration result")]
    CannotGetAddressFromRegistrationResult(#[from] chain_addr::Error),
    #[error("cannot get slot no from registration result")]
    CannotGetSlotNoFromRegistrationResult,
    #[error(transparent)]
    Bech32(#[from] chain_crypto::bech32::Error),
}
