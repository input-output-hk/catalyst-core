cfg_if::cfg_if! {
    if #[cfg(test)] {
        pub mod setup;
        pub mod public;
        pub mod private;
        pub mod non_functional;
    }
}

use thiserror::Error;

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
    BLANK = 0,
    YES = 1,
    NO = 2,
}
