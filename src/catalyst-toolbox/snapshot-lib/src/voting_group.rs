use crate::{voting_key::IdentifierDef, VotingGroup};
use jormungandr_lib::crypto::account::Identifier;
use std::{collections::HashSet, path::Path};
use thiserror::Error;

pub const DEFAULT_DIRECT_VOTER_GROUP: &str = "direct";
pub const DEFAULT_REPRESENTATIVE_GROUP: &str = "rep";

pub trait VotingGroupAssigner {
    fn assign(&self, vk: &Identifier) -> VotingGroup;
}

pub struct RepsVotersAssigner {
    direct_voters: VotingGroup,
    reps: VotingGroup,
    repsdb: HashSet<Identifier>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read reps file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse reps file: {0}")]
    Serde(#[from] serde_json::Error),
}

impl RepsVotersAssigner {
    pub fn new(direct_voters: VotingGroup, reps: VotingGroup) -> Self {
        Self {
            direct_voters,
            reps,
            repsdb: HashSet::new(),
        }
    }

    pub fn new_from_reps_file(
        direct_voters: VotingGroup,
        reps: VotingGroup,
        file_path: &Path,
    ) -> Result<Self, Error> {
        let f = std::fs::File::open(file_path)?;
        let repsdb_raw: Vec<IdentifierDef> = serde_json::from_reader(f)?;
        let repsdb = repsdb_raw.into_iter().map(|i| i.0).collect();

        Ok(Self {
            direct_voters,
            reps,
            repsdb,
        })
    }

    #[cfg(feature = "test-api")]
    pub fn new_from_repsdb(
        direct_voters: VotingGroup,
        reps: VotingGroup,
        repsdb: HashSet<Identifier>,
    ) -> Self {
        Self {
            direct_voters,
            reps,
            repsdb,
        }
    }
}

impl VotingGroupAssigner for RepsVotersAssigner {
    fn assign(&self, vk: &Identifier) -> VotingGroup {
        if self.repsdb.contains(vk) {
            self.reps.clone()
        } else {
            self.direct_voters.clone()
        }
    }
}

#[cfg(any(test, feature = "test-api", feature = "proptest"))]
impl<F> VotingGroupAssigner for F
where
    F: Fn(&Identifier) -> VotingGroup,
{
    fn assign(&self, vk: &Identifier) -> VotingGroup {
        self(vk)
    }
}
