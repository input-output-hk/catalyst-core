mod assert;
pub mod load;
mod mainnet;
pub mod registration;
mod rewards;
pub mod snapshot;
mod static_data;
mod vote_plan_status;
mod wallet;

pub use assert::*;
pub use mainnet::MainnetWallet;
pub use rewards::{funded_proposals, VotesRegistry};
pub use static_data::SnapshotExtensions;
use thiserror::Error;
pub use vote_plan_status::{CastedVote, VotePlanStatusProvider};
pub use wallet::{iapyx_from_mainnet, iapyx_from_qr, iapyx_from_secret_key};

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
