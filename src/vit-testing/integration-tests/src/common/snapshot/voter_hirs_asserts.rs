use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use voting_hir::VoterHIR;

pub trait VoterHIRAsserts {
    fn assert_contains_voting_key_and_value(&self, identifier: &Identifier, value: Value);
    fn assert_not_contain_voting_key(&self, identifier: &Identifier);
}

impl VoterHIRAsserts for Vec<VoterHIR> {
    fn assert_contains_voting_key_and_value(&self, identifier: &Identifier, value: Value) {
        assert!(self
            .iter()
            .any(|entry| { &entry.voting_key == identifier && entry.voting_power == value }));
    }

    fn assert_not_contain_voting_key(&self, identifier: &Identifier) {
        assert!(!self.iter().any(|entry| &entry.voting_key == identifier));
    }
}
