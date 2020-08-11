use crate::{ledger::governance::GovernanceAcceptanceCriteria, value::Value};
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use imhamt::Hamt;
use std::collections::{
    hash_map::{DefaultHasher, Entry},
    HashMap,
};
use typed_bytes::ByteBuilder;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParametersGovernanceAction {
    NoOp,
    RewardAdd { value: Value },
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ParametersGovernanceActionType {
    NoOp,
    RewardAdd,
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct ParametersGovernance {
    acceptance_criteria_per_action:
        Hamt<DefaultHasher, ParametersGovernanceActionType, GovernanceAcceptanceCriteria>,

    default_acceptance_criteria: GovernanceAcceptanceCriteria,

    logs: HashMap<ParametersGovernanceActionType, ParametersGovernanceAction>,
}

impl ParametersGovernanceAction {
    pub fn to_type(&self) -> ParametersGovernanceActionType {
        match self {
            Self::NoOp => ParametersGovernanceActionType::NoOp,
            Self::RewardAdd { .. } => ParametersGovernanceActionType::RewardAdd,
        }
    }

    pub(crate) fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        match self {
            Self::NoOp => bb.u8(0),
            Self::RewardAdd { value } => bb.u8(1).u64(value.0),
        }
    }
}

impl ParametersGovernance {
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
        action: ParametersGovernanceActionType,
        criteria: GovernanceAcceptanceCriteria,
    ) {
        self.acceptance_criteria_per_action = self
            .acceptance_criteria_per_action
            .insert_or_update_simple(action, criteria.clone(), |_| Some(criteria));
    }

    pub fn acceptance_criteria_for(
        &self,
        action: ParametersGovernanceActionType,
    ) -> &GovernanceAcceptanceCriteria {
        self.acceptance_criteria_per_action
            .lookup(&action)
            .unwrap_or_else(|| self.default_acceptance_criteria())
    }

    pub fn logs(&self) -> impl Iterator<Item = &ParametersGovernanceAction> {
        self.logs.values()
    }

    pub fn logs_clear(&mut self) {
        self.logs.clear()
    }

    /// register a new action
    pub fn logs_register(&mut self, action: ParametersGovernanceAction) -> Result<(), ()> {
        let entry = self.logs.entry(action.to_type());

        match entry {
            Entry::Vacant(vacant) => {
                vacant.insert(action);
                Ok(())
            }
            _ => Err(()),
        }
    }
}

/* Ser/De ******************************************************************* */

impl Readable for ParametersGovernanceAction {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        match buf.get_u8()? {
            0 => Ok(Self::NoOp),
            1 => {
                let value = Value::read(buf)?;
                Ok(Self::RewardAdd { value })
            }
            t => Err(ReadError::UnknownTag(t as u32)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{ParametersGovernance, ParametersGovernanceAction, ParametersGovernanceActionType};
    use crate::{ledger::governance::GovernanceAcceptanceCriteria, value::Value, vote::Choice};
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for ParametersGovernanceActionType {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let option = u8::arbitrary(g) % 2;
            match option {
                0 => ParametersGovernanceActionType::NoOp,
                1 => ParametersGovernanceActionType::RewardAdd,
                _ => unreachable!(),
            }
        }
    }

    impl Arbitrary for ParametersGovernanceAction {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let option = u8::arbitrary(g) % 2;
            match option {
                0 => ParametersGovernanceAction::NoOp,
                1 => ParametersGovernanceAction::RewardAdd {
                    value: Arbitrary::arbitrary(g),
                },
                _ => unreachable!(),
            }
        }
    }

    #[test]
    pub fn parameters_to_type() {
        let action = ParametersGovernanceAction::NoOp;
        assert_eq!(action.to_type(), ParametersGovernanceActionType::NoOp);

        let action = ParametersGovernanceAction::RewardAdd { value: Value(10) };
        assert_eq!(action.to_type(), ParametersGovernanceActionType::RewardAdd);
    }

    #[test]
    pub fn parameters_governance_set_default_acceptance_criteria() {
        let mut governance = ParametersGovernance::new();
        let new_governance_criteria = some_new_governance_criteria();
        governance.set_default_acceptance_criteria(new_governance_criteria.clone());

        assert_eq!(
            *governance.default_acceptance_criteria(),
            new_governance_criteria
        );
    }

    #[quickcheck]
    pub fn parameters_governance_set_acceptance_criteria(
        action_type: ParametersGovernanceActionType,
    ) {
        let mut governance = ParametersGovernance::new();
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

    #[quickcheck]
    pub fn parameters_governance_logs(action_type: ParametersGovernanceAction) {
        let mut governance = ParametersGovernance::new();
        assert!(governance.logs_register(action_type.clone()).is_ok());
        assert!(governance.logs().any(|x| *x == action_type));
        governance.logs_clear();

        assert!(!governance.logs().any(|x| *x == action_type));
        assert!(governance.logs().count() == 0);
    }
}
