//! define how to rule over the blockchain
//!

mod parameters;
mod treasury;

pub use self::{
    parameters::{
        ParametersGovernance, ParametersGovernanceAction, ParametersGovernanceActionType,
    },
    treasury::{TreasuryGovernance, TreasuryGovernanceAction, TreasuryGovernanceActionType},
};
use crate::{rewards::Ratio, vote::Choice};
use std::num::NonZeroU64;

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Governance {
    pub treasury: treasury::TreasuryGovernance,
    pub parameters: parameters::ParametersGovernance,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GovernanceAcceptanceCriteria {
    pub minimum_stake_participation: Option<Ratio>,
    pub minimum_approval: Option<Ratio>,
    pub choice: Choice,
}

impl Default for GovernanceAcceptanceCriteria {
    fn default() -> Self {
        const CENT: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(100) };

        Self {
            minimum_stake_participation: Some(Ratio {
                numerator: 30,
                denominator: CENT,
            }),
            minimum_approval: Some(Ratio {
                numerator: 50,
                denominator: CENT,
            }),
            choice: Choice::new(1),
        }
    }
}
