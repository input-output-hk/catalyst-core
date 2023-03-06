use self::{
    fund::{Fund, FundDb, FundIDs},
    snapshot::{Delegator, SnapshotDb, SnapshotVersions, Voter},
};

pub mod fund;
pub mod snapshot;

pub trait DB: SnapshotDb + FundDb {}

#[derive(Clone, Default)]
pub struct MockedDB {
    voter: Voter,
    delegator: Delegator,
    snapshot_versions: SnapshotVersions,
    fund: Fund,
    fund_ids: FundIDs,
}

impl SnapshotDb for MockedDB {
    fn get_snapshot_versions(&self) -> SnapshotVersions {
        self.snapshot_versions.clone()
    }
    fn get_voter(&self, _event: String, _voting_key: String) -> Voter {
        self.voter.clone()
    }
    fn get_delegator(&self, _event: String, _stake_public_key: String) -> Delegator {
        self.delegator.clone()
    }
}

impl FundDb for MockedDB {
    fn get_current_fund(&self) -> Fund {
        self.fund.clone()
    }
    fn get_fund_by_id(&self, _id: i32) -> Fund {
        self.fund.clone()
    }
    fn get_fund_ids(&self) -> FundIDs {
        self.fund_ids.clone()
    }
}

impl DB for MockedDB {}
