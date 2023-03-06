use self::{
    fund::{Fund, FundDb},
    proposal::{Proposal, ProposalDb},
    snapshot::{Delegator, SnapshotDb, Voter},
};

pub mod fund;
pub mod proposal;
pub mod snapshot;

pub trait DB: SnapshotDb + FundDb + ProposalDb {}

#[derive(Clone)]
pub struct MockedDB;

impl SnapshotDb for MockedDB {
    fn get_snapshot_versions(&self) -> Vec<String> {
        Default::default()
    }
    fn get_voter(&self, _event: String, _voting_key: String) -> Voter {
        Default::default()
    }
    fn get_delegator(&self, _event: String, _stake_public_key: String) -> Delegator {
        Default::default()
    }
}

impl FundDb for MockedDB {
    fn get_current_fund(&self) -> Fund {
        Default::default()
    }
    fn get_fund_by_id(&self, _id: i32) -> Fund {
        Default::default()
    }
    fn get_fund_ids(&self) -> Vec<i32> {
        Default::default()
    }
}

impl ProposalDb for MockedDB {
    fn get_proposals_by_voter_group_id(&self, _voter_group_id: String) -> Vec<Proposal> {
        Default::default()
    }
    fn get_proposal_by_and_by_voter_group_id(&self, _id: i32, _voter_group_id: String) -> Proposal {
        Default::default()
    }
}

impl DB for MockedDB {}
