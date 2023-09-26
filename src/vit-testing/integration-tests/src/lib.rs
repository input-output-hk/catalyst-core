extern crate core;
cfg_if::cfg_if! {
    if #[cfg(test)] {
        #[allow(clippy::all)]
        pub mod common;
        #[allow(clippy::all)]
        pub mod component;
        #[allow(clippy::all)]
        pub mod non_functional;
        #[allow(clippy::all)]
        pub mod integration;
        #[allow(clippy::all)]
        pub mod e2e;
    }
}

use jormungandr_automation::{jormungandr::JormungandrRest, testing::time};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("vitup error")]
    VitupError(#[from] vitup::error::Error),
    #[error("verification error")]
    VerificationError(#[from] jormungandr_automation::testing::VerificationError),
    #[error("sender error")]
    FragmentSenderError(#[from] thor::FragmentSenderError),
    #[error("iapyx error")]
    IapyxError(#[from] iapyx::ControllerError),
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Vote {
    Yes = 0,
    No = 1,
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

    pub fn wait_for_tally_start(self, rest: JormungandrRest) {
        time::wait_for_epoch(self.tally_start, rest);
    }

    pub fn wait_for_tally_end(self, rest: JormungandrRest) {
        time::wait_for_epoch(self.tally_end, rest);
    }
}
