pub mod c;
mod error;
mod wallet;

pub use self::{
    c::Conversion,
    error::{Error, ErrorCode, ErrorKind, Result},
    wallet::Wallet,
};
