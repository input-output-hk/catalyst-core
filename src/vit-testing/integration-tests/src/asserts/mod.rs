use chain_impl_mockchain::key::Hash;
use jormungandr_lib::interfaces::{Tally, VotePlanStatus};
use std::str::FromStr;

pub struct VotePlanStatusAssert {
    vote_plan_statuses: Vec<VotePlanStatus>,
}

impl From<Vec<VotePlanStatus>> for VotePlanStatusAssert {
    fn from(vote_plan_statuses: Vec<VotePlanStatus>) -> Self {
        VotePlanStatusAssert { vote_plan_statuses }
    }
}

impl VotePlanStatusAssert {
    pub fn assert_all_proposals_are_tallied(&self) {
        for vote_plan_status in self.vote_plan_statuses.iter() {
            self.assert_all_proposals_in_voteplan_are_tallied(&vote_plan_status);
        }
    }

    pub fn assert_all_proposals_in_voteplan_are_tallied(&self, vote_plan_status: &VotePlanStatus) {
        for proposal in vote_plan_status.proposals.iter() {
            assert!(
                proposal.tally.is_some(),
                "Proposal is not tallied {:?}",
                proposal
            );
        }
    }

    pub fn assert_proposal_tally(&self, vote_plan_id: String, index: u8, expected: Vec<u64>) {
        let vote_plan_status = self
            .vote_plan_statuses
            .iter()
            .find(|c_vote_plan| c_vote_plan.id == Hash::from_str(&vote_plan_id).unwrap().into())
            .unwrap();

        let tally = vote_plan_status
            .proposals
            .iter()
            .find(|x| x.index == index)
            .unwrap()
            .tally
            .as_ref()
            .unwrap();

        match tally {
            Tally::Public { result } => assert_eq!(expected, result.results()),
            Tally::Private { state: _ } => unimplemented!(),
        }
    }
}
