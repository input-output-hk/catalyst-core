mod batch;
mod single;

pub use batch::BatchWalletRequestGen;
pub use single::WalletRequestGen;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestGenError {
    #[error("wallet error")]
    Wallet(#[from] crate::wallet::Error),
    #[error("wallet error")]
    Backend(#[from] crate::backend::WalletBackendError),
    #[error("pin read error")]
    MultiController(#[from] crate::load::MultiControllerError),
}
