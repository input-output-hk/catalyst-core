pub mod c;
mod conversion;
mod error;
mod vote;
mod wallet;

pub use self::{
    conversion::Conversion,
    error::{Error, ErrorCode, ErrorKind, Result},
    vote::Proposal,
    wallet::Wallet,
};
pub use ::wallet::Settings;
pub use chain_impl_mockchain::{
    value::Value,
    vote::{Choice, Options, PayloadType},
};
pub use vote::VOTE_PLAN_ID_LENGTH;
