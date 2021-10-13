mod backend;
pub mod load;
pub mod registration;
pub mod snapshot;
mod wallet;
pub use backend::*;

use thiserror::Error;
pub use wallet::{iapyx_from_qr, iapyx_from_secret_key};

#[derive(Debug, Error)]
pub enum Error {
    #[error("vitup error")]
    VitupError(#[from] vitup::error::Error),
    #[error("node error")]
    NodeError(#[from] jormungandr_scenario_tests::node::Error),
    #[error("verification error")]
    VerificationError(#[from] jormungandr_testing_utils::testing::VerificationError),
    #[error("sender error")]
    FragmentSenderError(#[from] jormungandr_testing_utils::testing::FragmentSenderError),
    #[error("scenario error")]
    ScenarioError(#[from] jormungandr_scenario_tests::scenario::Error),
    #[error("iapyx error")]
    IapyxError(#[from] iapyx::ControllerError),
}

#[allow(dead_code)]
pub enum Vote {
    Blank = 0,
    Yes = 1,
    No = 2,
}
