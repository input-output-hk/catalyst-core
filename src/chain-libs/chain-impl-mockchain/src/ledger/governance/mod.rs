//! define how to rule over the blockchain
//!

mod treasury;

pub use self::treasury::{
    TreasuryGovernance, TreasuryGovernanceAcceptanceCriteria, TreasuryGovernanceAction,
};
