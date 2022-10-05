use jormungandr_lib::crypto::account::Identifier;
use snapshot_lib::voting_group::RepsVotersAssigner;
use std::collections::HashSet;

pub const DIRECT_VOTING_GROUP: &str = "direct";
pub const REP_VOTING_GROUP: &str = "rep";

pub trait RepsVoterAssignerSource {
    fn into_reps_voter_assigner(self) -> RepsVotersAssigner;
}

impl RepsVoterAssignerSource for HashSet<Identifier> {
    fn into_reps_voter_assigner(self) -> RepsVotersAssigner {
        RepsVotersAssigner::new_from_repsdb(
            DIRECT_VOTING_GROUP.to_string(),
            REP_VOTING_GROUP.to_string(),
            self,
        )
        .unwrap()
    }
}

pub fn empty_assigner() -> RepsVotersAssigner {
    HashSet::new().into_reps_voter_assigner()
}
