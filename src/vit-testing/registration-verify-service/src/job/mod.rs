mod info;

use crate::config::NetworkType;
use crate::context::ContextLock;
use crate::context::{Context, Step};
use crate::job::info::Assert;
use crate::job::info::Checks;
use crate::job::info::RegistrationInfo;
use crate::job::info::SnapshotInfo;
use crate::request::Request;
use chain_addr::AddressReadable;
use chain_addr::{Address, Discrimination, Kind};
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use iapyx::PinReadMode;
use iapyx::QrReader;
pub use info::JobOutputInfo;
use jormungandr_integration_tests::common::jcli::JCli;
use jortestkit::prelude::read_file;
use jortestkit::prelude::ProcessOutput;
use serde::{Deserialize, Serialize};
use snapshot_trigger_service::client::do_snapshot;
use snapshot_trigger_service::config::JobParameters;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;

pub struct RegistrationVerifyJobBuilder {
    job: RegistrationVerifyJob,
}

impl RegistrationVerifyJobBuilder {
    pub fn new() -> Self {
        Self {
            job: Default::default(),
        }
    }

    pub fn with_jcli<P: AsRef<Path>>(mut self, jcli: P) -> Self {
        self.job.jcli = jcli.as_ref().to_path_buf();
        self
    }

    pub fn with_snapshot_token<S: Into<String>>(mut self, snapshot_token: S) -> Self {
        self.job.snapshot_token = snapshot_token.into();
        self
    }

    pub fn with_snapshot_address<S: Into<String>>(mut self, snapshot_address: S) -> Self {
        self.job.snapshot_address = snapshot_address.into();
        self
    }

    pub fn with_network(mut self, network: NetworkType) -> Self {
        self.job.network = network;
        self
    }

    pub fn with_working_dir<P: AsRef<Path>>(mut self, working_dir: P) -> Self {
        self.job.working_dir = working_dir.as_ref().to_path_buf();
        self
    }

    pub fn build(self) -> RegistrationVerifyJob {
        self.job
    }
}

pub struct RegistrationVerifyJob {
    jcli: PathBuf,
    network: NetworkType,
    snapshot_token: String,
    snapshot_address: String,
    working_dir: PathBuf,
}

impl Default for RegistrationVerifyJobBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RegistrationVerifyJob {
    fn default() -> Self {
        Self {
            jcli: PathBuf::from_str("jcli").unwrap(),
            snapshot_token: "".to_string(),
            snapshot_address: "".to_string(),
            network: NetworkType::Mainnet,
            working_dir: PathBuf::from_str(".").unwrap(),
        }
    }
}

impl RegistrationVerifyJob {
    pub fn start(&self, request: Request, context: ContextLock) -> Result<JobOutputInfo, Error> {
        let jobs_params = JobParameters {
            slot_no: Some(request.slot_no),
            threshold: request.threshold,
        };

        context
            .lock()
            .unwrap()
            .state_mut()
            .update_running_step(Step::RunningSnapshot);

        /*    let snapshot_result = do_snapshot(
            jobs_params,
            self.snapshot_token.to_string(),
            self.snapshot_address.to_string(),
        )?;*/

        context
            .lock()
            .unwrap()
            .state_mut()
            .update_running_step(Step::ExtractingQRCode);

        let address = match QrReader::new(PinReadMode::Global(request.pin.clone()))
            .read_qr_from_bytes(request.qr.clone())
        {
            Ok(secret_key) => {
                checks.push(Assert::Passed(format!(
                    "succesfully read qr code:  ('{}')",
                    address_readable
                )));
                chain_addr::Address(
                    Discrimination::Production,
                    Kind::Account(secret_key.to_public()),
                )
            }
            Err(err) => {
                checks.push(Assert::Failed(format!(
                    "malformed qr:  ('{}')",
                    err.to_string()
                )));
            }
        };

        let mut checks: Checks = Default::default();

        //    let entry = snapshot_result.by_address(&address)?;
        let address_readable = AddressReadable::from_address("ca", &address);
        println!("succesfully read qr code:  ('{}')", address_readable);

        context
            .lock()
            .unwrap()
            .state_mut()
            .update_running_step(Step::VerifyingRegistration);

        /*
                match entry {
                    Some(entry) => {
                        checks.push(Assert::Passed(format!(
                            "entry found in snapshot ('{}') with funds: '{}'",
                            address_readable, entry.value
                        )));
                        checks.push(Assert::Passed(format!(
                            "adress is above threshold ('{}') with funds: '{}'",
                            request.expected_funds, entry.value
                        )));
                        checks.push(Assert::from_eq(
                            entry.value,
                            request.expected_funds.into(),
                            format!("correct funds amount '{}'", request.expected_funds),
                            format!(
                                "wrong funds amount '{}' != '{}'",
                                entry.value, request.expected_funds
                            ),
                        ));
                    }
                    None => {
                        checks.push(Assert::Failed(format!(
                            "entry not found in snapshot {}",
                            address_readable
                        )));
                        checks.push(Assert::Failed(format!(
                            "entry not found in snapshot {}",
                            address_readable
                        )));
                        checks.push(Assert::Failed(format!(
                            "entry not found in snapshot {}",
                            address_readable
                        )));
                    }
                }
        */
        Ok(JobOutputInfo {
            checks,
            registration: RegistrationInfo {
                expected_funds: request.expected_funds,
            },
            snapshot: SnapshotInfo {
                threshold: request.threshold,
                slot_no: request.slot_no,
            },
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
    #[error("cannot parse voter-registration output: {0:?}")]
    CannotParseVoterRegistrationOutput(Vec<String>),
    #[error("cannot parse cardano cli output: {0:?}")]
    CannotParseCardanoCliOutput(Vec<String>),
    #[error("snapshot trigger service")]
    SnapshotTriggerService(#[from] snapshot_trigger_service::client::Error),
    #[error("snapshot trigger service")]
    PinReadError(#[from] iapyx::PinReadError),
}
