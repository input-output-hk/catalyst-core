//! define how to rule over the blockchain
//!

mod treasury;

pub use self::treasury::{
    TreasuryGovernance, TreasuryGovernanceAcceptanceCriteria, TreasuryGovernanceAction,
    TreasuryGovernanceActionType,
};

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Governance {
    pub treasury: treasury::TreasuryGovernance,
}
