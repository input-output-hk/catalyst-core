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
    #[error("verification error")]
    VerificationError(#[from] jormungandr_automation::testing::VerificationError),
    #[error("sender error")]
    FragmentSenderError(#[from] thor::FragmentSenderError),
    #[error("iapyx error")]
    IapyxError(#[from] iapyx::ControllerError),
}

#[allow(dead_code)]
pub enum Vote {
    Yes = 0,
    No = 1,
}
