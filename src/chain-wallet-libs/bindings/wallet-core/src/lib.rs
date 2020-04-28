pub mod c;
mod conversion;
mod error;
mod wallet;

pub use self::{
    conversion::Conversion,
    error::{Error, ErrorCode, ErrorKind, Result},
    wallet::Wallet,
};
pub use ::wallet::Settings;
pub use chain_impl_mockchain::value::Value;
