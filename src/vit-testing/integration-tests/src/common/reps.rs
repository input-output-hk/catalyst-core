use jormungandr_lib::crypto::account::Identifier;
use snapshot_lib::voting_group::RepsVotersAssigner;
use std::collections::HashSet;
use vitup::config::{DIRECT_VOTING_GROUP, REP_VOTING_GROUP};

pub trait RepsVoterAssignerSource {
    fn into_reps_voter_assigner(self) -> RepsVotersAssigner;
}

impl RepsVoterAssignerSource for HashSet<Identifier> {
    fn into_reps_voter_assigner(self) -> RepsVotersAssigner {
        RepsVotersAssigner::new(
            DIRECT_VOTING_GROUP.to_string(),
            REP_VOTING_GROUP.to_string(),
            self.into(),
        )
    }
}

pub fn empty_assigner() -> RepsVotersAssigner {
    HashSet::new().into_reps_voter_assigner()
}
