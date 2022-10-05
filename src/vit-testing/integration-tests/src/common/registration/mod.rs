mod controller;
mod remote;
mod result_assert;
mod starter;

use assert_fs::TempDir;
pub use controller::RegistrationServiceController;
pub use result_assert::RegistrationResultAsserts;
pub use starter::RegistrationServiceStarter;

use registration_service::client::do_registration as do_registration_internal;
use registration_service::client::rest::RegistrationRestClient;
use registration_service::client::{Error as RegistrationClientError, RegistrationResult};
use registration_service::request::Request;
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
    let legacy_skey = std::env::var("VOTE_SKEY").ok();

    let registration_request = Request {
        payment_skey,
        payment_vkey,
        stake_skey,
        stake_vkey,
        legacy_skey,
        delegation_1: None,
        delegation_2: None,
        delegation_3: None,
    };

    let registration_client =
        RegistrationRestClient::new_with_token(registration_token, registration_address);

    do_registration_internal(registration_request, &registration_client, temp_dir)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("spawn command.rs")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RegistrationClient(#[from] RegistrationClientError),
    #[error(transparent)]
    Config(#[from] registration_service::config::Error),
    #[error("cannot bootstrap registration service on port {0}")]
    Bootstrap(u16),
}
