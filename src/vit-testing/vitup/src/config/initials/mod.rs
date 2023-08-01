mod block0;

pub use block0::{Initial as Block0Initial, Initials as Block0Initials};
use mainnet_lib::Initials as SnapshotInitials;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Initials {
    #[serde(default)]
    pub snapshot: Option<SnapshotInitials>,
    #[serde(default)]
    pub block0: Block0Initials,
}

pub const DIRECT_VOTING_GROUP: &str = "direct";
pub const REP_VOTING_GROUP: &str = "rep";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum Role {
    Representative,
    #[default]
    Voter,
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        Role::from_str(&s).map_err(|e| de::Error::custom(e.to_string()))
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string().to_lowercase())
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Representative => write!(f, "{}", REP_VOTING_GROUP),
            Role::Voter => write!(f, "{}", DIRECT_VOTING_GROUP),
        }
    }
}

impl FromStr for Role {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            REP_VOTING_GROUP | "reps" | "dreps" | "representative" => Ok(Role::Representative),
            DIRECT_VOTING_GROUP | "voter" => Ok(Role::Voter),
            _ => Err(Error::UnknownRole(s.to_string())),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown type of role: {0}")]
    UnknownRole(String),
}

#[cfg(test)]
mod test {
    use crate::config::{Role, DIRECT_VOTING_GROUP, REP_VOTING_GROUP};
    use std::str::FromStr;

    #[test]
    fn role_bijection_test() {
        let conversions = vec![
            (DIRECT_VOTING_GROUP, Role::Voter),
            (REP_VOTING_GROUP, Role::Representative),
            ("voter", Role::Voter),
            ("dreps", Role::Representative),
            ("rep", Role::Representative),
            ("dreps", Role::Representative),
        ];

        for (input, expected) in conversions {
            let role: Role = Role::from_str(input).unwrap();
            assert_eq!(role, expected)
        }
    }
}
