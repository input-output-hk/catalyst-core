use crate::{rewards::Ratio, value::Value};
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use imhamt::Hamt;
use std::{collections::hash_map::DefaultHasher, num::NonZeroU64};
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TreasuryGovernanceAction {
    TransferToRewards { value: Value },
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TreasuryGovernanceActionType {
    TransferToRewards,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TreasuryGovernanceAcceptanceCriteria {
    pub minimum_stake_participation: Option<Ratio>,
    pub minimum_approval: Option<Ratio>,
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct TreasuryGovernance {
    acceptance_criteria_per_action:
        Hamt<DefaultHasher, TreasuryGovernanceActionType, TreasuryGovernanceAcceptanceCriteria>,

    default_acceptance_criteria: TreasuryGovernanceAcceptanceCriteria,
}

impl TreasuryGovernanceAction {
    pub fn to_type(&self) -> TreasuryGovernanceActionType {
        match self {
            Self::TransferToRewards { .. } => TreasuryGovernanceActionType::TransferToRewards,
        }
    }

    pub(crate) fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        match self {
            Self::TransferToRewards { value } => bb.u8(1).u64(value.0),
        }
    }
}

impl TreasuryGovernance {
    pub fn new() -> Self {
        Self::default()
    }

    /// set the new default acceptance criteria
    ///
    /// this function does not do any allocation/drop and returns the previous
    /// default value.
    pub fn set_default_acceptance_criteria(
        &mut self,
        new: TreasuryGovernanceAcceptanceCriteria,
    ) -> TreasuryGovernanceAcceptanceCriteria {
        std::mem::replace(&mut self.default_acceptance_criteria, new)
    }

    /// get the default acceptance criteria
    ///
    /// This is the default criteria that will be used for any
    /// treasury governance action if a specific one is not set
    /// for that given governance action.
    pub fn default_acceptance_criteria(&self) -> &TreasuryGovernanceAcceptanceCriteria {
        &self.default_acceptance_criteria
    }

    pub fn set_acceptance_criteria(
        &mut self,
        action: TreasuryGovernanceActionType,
        criteria: TreasuryGovernanceAcceptanceCriteria,
    ) {
        self.acceptance_criteria_per_action = self
            .acceptance_criteria_per_action
            .insert_or_update_simple(action, criteria.clone(), |_| Some(criteria));
    }

    pub fn acceptance_criteria_for(
        &self,
        action: TreasuryGovernanceActionType,
    ) -> &TreasuryGovernanceAcceptanceCriteria {
        self.acceptance_criteria_per_action
            .lookup(&action)
            .unwrap_or_else(|| self.default_acceptance_criteria())
    }
}

impl Default for TreasuryGovernanceAcceptanceCriteria {
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
        }
    }
}

/* Ser/De ******************************************************************* */

impl Readable for TreasuryGovernanceAction {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        match buf.get_u8()? {
            1 => {
                let value = Value::read(buf)?;
                Ok(Self::TransferToRewards { value })
            }
            t => Err(ReadError::UnknownTag(t as u32)),
        }
    }
}
