cfg_if::cfg_if! {
    if #[cfg(test)] {
        pub mod setup;
        pub mod public;
        pub mod private;
        pub mod non_functional;
        pub mod asserts;
    }
}

use jormungandr_testing_utils::testing::node::time;
use jormungandr_testing_utils::testing::node::Explorer;
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
    Blank = 0,
    Yes = 1,
    No = 2,
}

#[allow(dead_code)]
pub struct VoteTiming {
    pub vote_start: u32,
    pub tally_start: u32,
    pub tally_end: u32,
}

impl VoteTiming {
    pub fn new(vote_start: u32, tally_start: u32, tally_end: u32) -> Self {
        Self {
            vote_start,
            tally_start,
            tally_end,
        }
    }

    pub fn wait_for_tally_start(self, explorer: Explorer) {
        time::wait_for_epoch(self.tally_start as u64, explorer);
    }

    pub fn wait_for_tally_end(self, explorer: Explorer) {
        time::wait_for_epoch(self.tally_end as u64, explorer);
    }
}
