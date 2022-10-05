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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Role {
    Representative,
    Voter,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Representative => write!(f, "dreps"),
            Role::Voter => write!(f, "direct"),
        }
    }
}

impl FromStr for Role {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "direct" {
            Ok(Role::Voter)
        } else if s.to_lowercase() == "dreps" {
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
