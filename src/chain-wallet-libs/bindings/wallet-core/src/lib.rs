pub mod c;
mod error;
mod vote;
mod wallet;

pub use self::{
    error::{Error, ErrorCode, ErrorKind, Result},
    vote::Proposal,
    wallet::Wallet,
};
pub use ::wallet::Settings;
pub use chain_impl_mockchain::{
    fragment::{Fragment, FragmentId},
    value::Value,
    vote::{Choice, Options, PayloadType},
};
pub use vote::{PayloadTypeConfig, VOTE_PLAN_ID_LENGTH};
