use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use catalyst_toolbox::kedqr::PinReadMode;
use iapyx::utils::qr::{Secret, SecretFromQrCode};
use jormungandr_automation::jcli::JCli;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use jortestkit::prelude::WaitBuilder;
use math::round;
use registration_service::{
    client::rest::RegistrationRestClient, context::State, request::Request,
};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub fn do_registration(temp_dir: &TempDir) -> RegistrationResult {
    let registration_token = std::env::var("REGISTRATION_TOKEN")
        .unwrap_or_else(|_| "REGISTRATION_TOKEN not defined".to_owned());
    let registration_address = std::env::var("REGISTRATION_ADDRESS")
        .unwrap_or_else(|_| "REGISTRATION_ADDRESS not defined".to_owned());
    let payment_skey =
        std::env::var("PAYMENT_SKEY").unwrap_or_else(|_| "PAYMENT_SKEY not defined".to_owned());
    let payment_vkey =
        std::env::var("PAYMENT_VKEY").unwrap_or_else(|_| "PAYMENT_VKEY not defined".to_owned());
    let stake_skey =
        std::env::var("STAKE_SKEY").unwrap_or_else(|_| "STAKE_SKEY not defined".to_owned());
    let stake_vkey =
        std::env::var("STAKE_VKEY").unwrap_or_else(|_| "STAKE_VKEY not defined".to_owned());
    let vote_skey = std::env::var("VOTE_SKEY").ok();

    let registration_client =
        RegistrationRestClient::new_with_token(registration_token, registration_address);

    let registration_request = Request {
        payment_skey,
        payment_vkey,
        stake_skey,
        stake_vkey,
        vote_skey,
    };

    let registration_job_id = registration_client.job_new(registration_request).unwrap();

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();
    println!("waiting for registration job");
    let registration_jobs_status = registration_client
        .wait_for_job_finish(registration_job_id.clone(), wait)
        .unwrap();
    println!("{:?}", registration_jobs_status);

    let qr_code_path = temp_dir.child("qr_code");
    std::fs::create_dir_all(qr_code_path.path()).unwrap();

    let qr_code = registration_client
        .download_qr(registration_job_id.clone(), qr_code_path.path())
        .unwrap();
    let voting_key_sk = registration_client
        .get_catalyst_sk(registration_job_id)
        .unwrap();

    RegistrationResult {
        status: registration_jobs_status,
        qr_code,
        voting_sk: voting_key_sk,
    }
}

#[derive(Debug)]
pub struct RegistrationResult {
    status: State,
    qr_code: PathBuf,
    voting_sk: String,
}

impl RegistrationResult {
    pub fn assert_status_is_finished(&self) {
        matches!(self.status, State::Finished { .. });
    }

    pub fn assert_qr_equals_to_sk(&self) {
        let bech32_key = Secret::from_file(
            &self.qr_code,
            PinReadMode::FromFileName(self.qr_code.to_path_buf()),
        )
        .expect("unable to read qr code")
        .to_bech32()
        .expect("unable to export key as bech32");

        assert_eq!(
            self.voting_sk, bech32_key,
            "secret key from qr is not equal to used during registration"
        );
    }

    pub fn status(&self) -> State {
        self.status.clone()
    }

    pub fn qr_code(&self) -> PathBuf {
        self.qr_code.clone()
    }

    pub fn pin(&self) -> String {
        let chars: Vec<char> = self
            .qr_code
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .rev()
            .take(4)
            .collect();
        chars.iter().rev().collect()
    }

    pub fn snapshot_entry(&self) -> Result<(Identifier, Value), Error> {
        Ok((self.identifier()?, self.funds_in_ada()?.into()))
    }

    pub fn print_snapshot_entry(&self) -> Result<(), Error> {
        println!(
            "[identifier: {}, funds:{}",
            self.identifier_as_str(),
            self.funds_in_ada()?
        );
        Ok(())
    }

    pub fn identifier_as_str(&self) -> String {
        let jcli = JCli::new(PathBuf::from_str("jcli").expect("jcli not found on env"));
        jcli.key().convert_to_public_string(&self.voting_sk)
    }

    pub fn identifier(&self) -> Result<Identifier, Error> {
        Ok(Identifier::from_str(&self.identifier_as_str())?)
    }

    pub fn slot_no(&self) -> Result<u64, Error> {
        match self.status() {
            State::Finished { info, .. } => Ok(info.slot_no),
            _ => Err(Error::CannotGetSlotNoFromRegistrationResult),
        }
    }

    pub fn funds_in_ada(&self) -> Result<u64, Error> {
        match self.status() {
            State::Finished { info, .. } => {
                let rounded = round::floor(info.funds as f64, -6);
                Ok((rounded as u64) / 1_000_000)
            }
            _ => Err(Error::CannotGetFundsFromRegistrationResult),
        }
    }

    pub fn funds_in_lovelace(&self) -> Result<u64, Error> {
        match self.status() {
            State::Finished { info, .. } => Ok(info.funds),
            _ => Err(Error::CannotGetFundsFromRegistrationResult),
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
