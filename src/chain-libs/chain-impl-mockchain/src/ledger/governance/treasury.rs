use crate::{ledger::governance::GovernanceAcceptanceCriteria, value::Value};
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use imhamt::Hamt;
use std::collections::hash_map::DefaultHasher;
use typed_bytes::ByteBuilder;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TreasuryGovernanceAction {
    NoOp,
    TransferToRewards { value: Value },
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TreasuryGovernanceActionType {
    NoOp,
    TransferToRewards,
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct TreasuryGovernance {
    acceptance_criteria_per_action:
        Hamt<DefaultHasher, TreasuryGovernanceActionType, GovernanceAcceptanceCriteria>,

    default_acceptance_criteria: GovernanceAcceptanceCriteria,
}

impl TreasuryGovernanceAction {
    pub fn to_type(&self) -> TreasuryGovernanceActionType {
        match self {
            Self::NoOp => TreasuryGovernanceActionType::NoOp,
            Self::TransferToRewards { .. } => TreasuryGovernanceActionType::TransferToRewards,
        }
    }

    pub(crate) fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        match self {
            Self::NoOp => bb.u8(0),
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
        new: GovernanceAcceptanceCriteria,
    ) -> GovernanceAcceptanceCriteria {
        std::mem::replace(&mut self.default_acceptance_criteria, new)
    }

    /// get the default acceptance criteria
    ///
    /// This is the default criteria that will be used for any
    /// treasury governance action if a specific one is not set
    /// for that given governance action.
    pub fn default_acceptance_criteria(&self) -> &GovernanceAcceptanceCriteria {
        &self.default_acceptance_criteria
    }

    pub fn set_acceptance_criteria(
        &mut self,
        action: TreasuryGovernanceActionType,
        criteria: GovernanceAcceptanceCriteria,
    ) {
        self.acceptance_criteria_per_action = self
            .acceptance_criteria_per_action
            .insert_or_update_simple(action, criteria.clone(), |_| Some(criteria));
    }

    pub fn acceptance_criteria_for(
        &self,
        action: TreasuryGovernanceActionType,
    ) -> &GovernanceAcceptanceCriteria {
        self.acceptance_criteria_per_action
            .lookup(&action)
            .unwrap_or_else(|| self.default_acceptance_criteria())
    }
}

/* Ser/De ******************************************************************* */

impl Readable for TreasuryGovernanceAction {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        match buf.get_u8()? {
            0 => Ok(Self::NoOp),
            1 => {
                let value = Value::read(buf)?;
                Ok(Self::TransferToRewards { value })
            }
            t => Err(ReadError::UnknownTag(t as u32)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{TreasuryGovernance, TreasuryGovernanceAction, TreasuryGovernanceActionType};
    use crate::{ledger::governance::GovernanceAcceptanceCriteria, value::Value, vote::Choice};
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for TreasuryGovernanceActionType {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let option = u8::arbitrary(g) % 2;
            match option {
                0 => TreasuryGovernanceActionType::NoOp,
                1 => TreasuryGovernanceActionType::TransferToRewards,
                _ => unreachable!(),
            }
        }
    }
    #[test]
    pub fn treasury_governance_to_type() {
        let action = TreasuryGovernanceAction::NoOp;
        assert_eq!(action.to_type(), TreasuryGovernanceActionType::NoOp);

        let action = TreasuryGovernanceAction::TransferToRewards { value: Value(10) };
        assert_eq!(
            action.to_type(),
            TreasuryGovernanceActionType::TransferToRewards
        );
    }

    #[test]
    pub fn treasury_governance_set_default_acceptance_criteria() {
        let mut governance = TreasuryGovernance::new();
        let new_governance_criteria = some_new_governance_criteria();
        governance.set_default_acceptance_criteria(new_governance_criteria.clone());

        assert_eq!(
            *governance.default_acceptance_criteria(),
            new_governance_criteria
        );
    }

    #[quickcheck]
    pub fn treasury_governance_set_acceptance_criteria(action_type: TreasuryGovernanceActionType) {
        let mut governance = TreasuryGovernance::new();
        let new_governance_criteria = some_new_governance_criteria();
        governance.set_acceptance_criteria(action_type, new_governance_criteria.clone());
        assert_eq!(
            *governance.acceptance_criteria_for(action_type),
            new_governance_criteria
        );
    }

    fn some_new_governance_criteria() -> GovernanceAcceptanceCriteria {
        let mut new_governance_criteria: GovernanceAcceptanceCriteria = Default::default();
        let new_option = Choice::new(20);
        new_governance_criteria.favorable = new_option;
        new_governance_criteria
    }
}
