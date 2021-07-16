mod info;

use crate::config::NetworkType;
use crate::context::{ContextLock, Step};
use crate::job::info::Assert;
use crate::job::info::Checks;
use crate::job::info::RegistrationInfo;
use crate::job::info::SnapshotInfo;
use crate::request::{Request, Source};
use chain_addr::Address;
use chain_addr::AddressReadable;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519;
use iapyx::PinReadMode;
use iapyx::QrReader;
pub use info::JobOutputInfo;
use snapshot_trigger_service::client::do_snapshot;
use snapshot_trigger_service::client::get_snapshot_from_history_by_id;
use snapshot_trigger_service::config::JobParameters;
use std::path::{Path, PathBuf};
use std::str::FromStr;
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

    pub fn with_network(mut self, network: NetworkType) -> Self {
        self.job.network = network;
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

    pub fn with_snapshot_initial_job_id(mut self, snapshot_job_id: Option<String>) -> Self {
        self.job.snapshot_job_id = snapshot_job_id;
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
    snapshot_token: String,
    snapshot_address: String,
    working_dir: PathBuf,
    snapshot_job_id: Option<String>,
    network: NetworkType,
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
            working_dir: PathBuf::from_str(".").unwrap(),
            snapshot_job_id: None,
            network: NetworkType::Testnet,
        }
    }
}

impl RegistrationVerifyJob {
    fn extract_address_from_request(
        &self,
        checks: &mut Checks,
        request: &Request,
    ) -> Option<Address> {
        match &request.source {
            Source::PublicKeyBytes(content) => {
                match chain_crypto::PublicKey::from_binary(&content) {
                    Ok(public_key) => Some(self.extract_address_from_public_key(
                        public_key,
                        checks,
                        "successfully parsed public key",
                    )),
                    Err(err) => {
                        checks.push(Assert::Failed(format!(
                            "malformed public key: '{}'",
                            err.to_string()
                        )));
                        None
                    }
                }
            }
            Source::Qr { pin, content } => {
                match QrReader::new(PinReadMode::Global(pin.clone()))
                    .read_qr_from_bytes(content.clone())
                {
                    Ok(secret_key) => Some(self.extract_address_from_public_key(
                        secret_key.to_public(),
                        checks,
                        "succesfully read qr code",
                    )),
                    Err(err) => {
                        checks.push(Assert::Failed(format!(
                            "malformed qr: '{}'",
                            err.to_string()
                        )));
                        None
                    }
                }
            }
        }
    }

    fn extract_address_from_public_key(
        &self,
        key: chain_crypto::PublicKey<Ed25519>,
        checks: &mut Checks,
        comment: &str,
    ) -> Address {
        let address = chain_addr::Address(Discrimination::Production, Kind::Account(key));
        checks.push(Assert::Passed(format!(
            "{}: '{}'",
            comment,
            AddressReadable::from_address("ca", &address)
        )));
        address
    }

    pub fn start(&self, request: Request, context: ContextLock) -> Result<JobOutputInfo, Error> {
        let jobs_params = JobParameters {
            slot_no: request.slot_no,
            threshold: request.threshold,
        };

        let registration = RegistrationInfo {
            expected_funds: request.expected_funds,
        };
        let snapshot = SnapshotInfo {
            threshold: request.threshold,
            slot_no: request.slot_no,
        };

        let mut checks: Checks = Default::default();

        context
            .lock()
            .unwrap()
            .state_mut()
            .update_running_step(Step::BuildingAddress);

        let address = match self.extract_address_from_request(&mut checks, &request) {
            Some(address) => address,
            None => {
                return Ok(JobOutputInfo {
                    checks,
                    registration,
                    snapshot,
                });
            }
        };

        context
            .lock()
            .unwrap()
            .state_mut()
            .update_running_step(Step::RunningSnapshot);

        let snapshot_result = match self.network {
            NetworkType::Testnet => do_snapshot(
                jobs_params,
                self.snapshot_token.to_string(),
                self.snapshot_address.to_string(),
            )?,
            NetworkType::Mainnet => {
                let context = context.lock().unwrap();

                let job_id = context
                    .config()
                    .snapshot_job_id
                    .as_ref()
                    .ok_or(Error::SnapshotJobIdNotDefined)?;

                get_snapshot_from_history_by_id(
                    job_id,
                    self.snapshot_token.to_string(),
                    self.snapshot_address.to_string(),
                )?
            }
        };

        let entry = snapshot_result.by_address(&address)?;
        let address_readable = AddressReadable::from_address("ca", &address);

        context
            .lock()
            .unwrap()
            .state_mut()
            .update_running_step(Step::VerifyingRegistration);

        match entry {
            Some(entry) => {
                checks.push(Assert::Passed(format!(
                    "wallet found in snapshot ('{}') with funds: '{}'",
                    address_readable, entry.value
                )));
                checks.push(Assert::Passed(format!(
                    "wallat is eligible for voting (has more than threshold '{}') with funds: '{}'",
                    request.threshold, entry.value
                )));
                checks.push(Assert::from_eq(
                    entry.value,
                    request.expected_funds.into(),
                    format!("correct wallet funds '{}'", request.expected_funds),
                    format!(
                        "incorrect wallet funds '{}' != '{}'",
                        entry.value, request.expected_funds
                    ),
                ));
            }
            None => {
                checks.push(Assert::Failed(format!(
                    "wallet not found in snapshot '{}' or has less than theshold: '{}'",
                    address_readable, request.threshold
                )));
            }
        }
        checks.calculate_passed();
        Ok(JobOutputInfo {
            checks,
            registration,
            snapshot,
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
    #[error("in mainnet mode snapshot is not runned on each request. Can't find job id of last snapshot job")]
    SnapshotJobIdNotDefined,
}
