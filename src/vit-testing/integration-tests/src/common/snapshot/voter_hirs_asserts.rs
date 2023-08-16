use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use snapshot_lib::registration::{Delegations, VotingRegistration};
use snapshot_lib::VoterHIR;

pub trait RegistrationAsserts {
    fn assert_contains_voting_key_and_value(&self, identifier: &Identifier, value: Value);
    fn assert_not_contain_voting_key(&self, identifier: &Identifier);
}

impl RegistrationAsserts for Vec<VotingRegistration> {
    fn assert_contains_voting_key_and_value(&self, identifier: &Identifier, value: Value) {
        assert!(self.iter().any(|x| {
            value == x.voting_power
                && match &x.delegations {
                    Delegations::New(hash_set) => {
                        hash_set.iter().any(|(id, _weight)| id == identifier)
                    }
                    Delegations::Legacy(id) => id == identifier,
                }
        }));
    }

    fn assert_not_contain_voting_key(&self, identifier: &Identifier) {
        assert!(!self.iter().any(|x| {
            match &x.delegations {
                Delegations::New(hash_set) => hash_set.iter().any(|(id, _weight)| id == identifier),
                Delegations::Legacy(id) => id == identifier,
            }
        }));
    }
}

impl RegistrationAsserts for Vec<VoterHIR> {
    fn assert_contains_voting_key_and_value(&self, identifier: &Identifier, value: Value) {
        assert!(self
            .iter()
            .any(|entry| { &entry.voting_key == identifier && entry.voting_power == value }));
    }

    fn assert_not_contain_voting_key(&self, identifier: &Identifier) {
        assert!(!self.iter().any(|entry| &entry.voting_key == identifier));
    }
}
