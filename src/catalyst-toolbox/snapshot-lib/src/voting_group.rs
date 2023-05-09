use crate::{Dreps, VotingGroup};
use jormungandr_lib::crypto::account::Identifier;
use std::collections::HashSet;

pub const DEFAULT_DIRECT_VOTER_GROUP: &str = "direct";
pub const DEFAULT_REPRESENTATIVE_GROUP: &str = "rep";

pub trait VotingGroupAssigner {
    fn assign(&self, vk: &Identifier) -> VotingGroup;
}

pub struct RepsVotersAssigner {
    direct_voters: VotingGroup,
    reps: VotingGroup,
    dreps: HashSet<Identifier>,
}

impl RepsVotersAssigner {
    pub fn new(direct_voters: VotingGroup, reps: VotingGroup, dreps: Dreps) -> Self {
        Self {
            direct_voters,
            reps,
            dreps: dreps.reps,
        }
    }
}

impl VotingGroupAssigner for RepsVotersAssigner {
    fn assign(&self, vk: &Identifier) -> VotingGroup {
        if self.dreps.contains(vk) {
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
