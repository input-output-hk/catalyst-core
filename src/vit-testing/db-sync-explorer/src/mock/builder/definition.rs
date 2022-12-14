#![allow(dead_code)]
// Most of the below structs will be created by deserializing incoming requests

use serde::{Deserialize, Serialize};
use std::slice::Iter;

pub type RepNameOrVotingKey = String;

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Target(Vec<(RepNameOrVotingKey, u32)>);

impl From<Vec<(&str, u32)>> for Target {
    fn from(inner: Vec<(&str, u32)>) -> Self {
        Target(
            inner
                .into_iter()
                .map(|(key, weight)| (key.to_string(), weight))
                .collect(),
        )
    }
}

impl Target {
    pub fn push(&mut self, value: (RepNameOrVotingKey, u32)) {
        self.0.push(value)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, (RepNameOrVotingKey, u32)> {
        self.0.iter()
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Registration {
    pub target: Target,
    pub slotno: u32,
}

pub fn registration() -> RegistrationBuilder {
    Default::default()
}

#[derive(Default)]
pub struct RegistrationBuilder {
    registration: Registration,
}

impl RegistrationBuilder {
    pub fn at_slot(mut self, slotno: u32) -> Self {
        self.registration.slotno = slotno;
        self
    }

    pub fn with_target(mut self, key: RepNameOrVotingKey, weight: u32) -> Self {
        self.registration.target.push((key, weight));
        self
    }

    pub fn with_targets(mut self, targets: Vec<(&str, u32)>) -> Self {
        self.registration.target = targets.into();
        self
    }
}

impl TryFrom<RegistrationBuilder> for Registration {
    type Error = Error;

    fn try_from(builder: RegistrationBuilder) -> Result<Self, Self::Error> {
        if builder.registration.target.is_empty() {
            return Err(Error::CannotBuildRegistration {
                registration: builder.registration,
                details: "empty registrations for generated delegator".to_string(),
            });
        }
        Ok(builder.registration)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Actor {
    ExternalDelegator {
        name: String,
        address: String,
    },
    GeneratedDelegator {
        name: String,
        registration: Registration,
        ada: u64,
    },
    ExternalRep {
        name: String,
        voting_key: String,
    },
    GeneratedRep {
        name: String,
        ada: u64,
    },
}

pub fn delegator(delegator: &str) -> DelegatorBuilder {
    DelegatorBuilder::new(delegator)
}

pub fn representative(representative: &str) -> RepresentativeBuilder {
    RepresentativeBuilder::new(representative)
}

pub struct DelegatorBuilder {
    name: String,
    ada: Option<u64>,
    address: Option<String>,
    registration: Registration,
}

impl DelegatorBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ada: None,
            address: None,
            registration: Default::default(),
        }
    }

    pub(crate) fn with_registration(mut self, reg: Registration) -> Self {
        self.registration = reg;
        self
    }

    pub(crate) fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    pub(crate) fn with_ada(mut self, ada: u64) -> Self {
        self.ada = Some(ada);
        self
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cannot build '{name}' actor instance, due to: {details}")]
    CannotBuildActor { name: String, details: String },

    #[error("cannot build '{registration:?}' actor instance, due to: {details}")]
    CannotBuildRegistration {
        registration: Registration,
        details: String,
    },
}

impl TryFrom<DelegatorBuilder> for Actor {
    type Error = Error;

    fn try_from(builder: DelegatorBuilder) -> Result<Self, Self::Error> {
        if let Some(ada) = builder.ada {
            if builder.registration.target.is_empty() {
                return Err(Error::CannotBuildActor {
                    name: builder.name,
                    details: "empty registrations for generated delegator".to_string(),
                });
            }

            Ok(Actor::GeneratedDelegator {
                name: builder.name,
                registration: builder.registration,
                ada,
            })
        } else if let Some(address) = builder.address {
            Ok(Actor::ExternalDelegator {
                name: builder.name,
                address,
            })
        } else {
            Err(Error::CannotBuildActor {
                name: builder.name.clone(),
                details: "no address defined for external delegator".to_string(),
            })
        }
    }
}

impl TryFrom<RepresentativeBuilder> for Actor {
    type Error = Error;

    fn try_from(builder: RepresentativeBuilder) -> Result<Self, Self::Error> {
        if let Some(ada) = builder.ada {
            Ok(Actor::GeneratedRep {
                name: builder.name,
                ada,
            })
        } else if let Some(voting_key) = builder.voting_key {
                Ok(Actor::ExternalRep {
                    name: builder.name,
                    voting_key,
                })
            } else {
            Err(Error::CannotBuildActor {
                name: builder.name.clone(),
                details: "no voting ke defined for external representative".to_string(),
            })
        }
    }
}

pub struct RepresentativeBuilder {
    name: String,
    ada: Option<u64>,
    voting_key: Option<String>,
}

impl RepresentativeBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ada: None,
            voting_key: None,
        }
    }

    pub(crate) fn with_key(mut self, key: impl Into<String>) -> Self {
        self.voting_key = Some(key.into());
        self
    }

    pub(crate) fn with_ada(mut self, ada: u64) -> Self {
        self.ada = Some(ada);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::builder::definition::{delegator, representative, Registration};
    use crate::mock::Actor;

    #[test]
    pub fn test() {
        let actors: Vec<Actor> = vec![
            representative("alice").with_ada(1000).try_into().unwrap(),
            representative("bob").with_key("").try_into().unwrap(),
            delegator("clarice")
                .with_registration(Registration {
                    target: vec![("bob", 1), ("alice", 1)].into(),
                    slotno: 1,
                })
                .try_into()
                .unwrap(),
            delegator("david")
                .with_address("testadd")
                .try_into()
                .unwrap(),
        ];

        assert_eq!(
            Actor::GeneratedRep {
                name: "alice".to_string(),
                ada: 1000
            },
            actors[0]
        );
        assert_eq!(
            actors[0],
            Actor::ExternalRep {
                name: "bob".to_string(),
                voting_key: "".to_string()
            }
        );
        assert_eq!(
            actors[0],
            Actor::GeneratedDelegator {
                name: "clarice".to_string(),
                registration: Registration {
                    target: vec![("bob", 1), ("alice", 1)].into(),
                    slotno: 1
                },
                ada: 0
            }
        );
        assert_eq!(
            actors[0],
            Actor::ExternalDelegator {
                name: "david".to_string(),
                address: "testadd".to_string()
            }
        );
    }
}
