mod block0;
mod snapshot;

pub use block0::{Initial as Block0Initial, Initials as Block0Initials};
use serde::{Deserialize, Serialize};
pub use snapshot::{
    Error as SnapshotError, Initial as SnapshotInitial, Initials as SnapshotInitials,
};
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Role {
    Representative,
    Voter,
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
        if s.to_lowercase() == DIRECT_VOTING_GROUP {
            Ok(Role::Voter)
        } else if s.to_lowercase() == REP_VOTING_GROUP {
            Ok(Role::Representative)
        } else {
            Err(Error::UnknownRole(s.to_string()))
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown type of role: {0}")]
    UnknownRole(String),
}

impl Default for Role {
    fn default() -> Self {
        Role::Voter
    }
}
