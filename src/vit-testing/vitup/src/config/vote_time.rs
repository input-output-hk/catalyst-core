use crate::Result;
use chrono::NaiveDateTime;
use jormungandr_testing_utils::testing::node::time;
use jormungandr_testing_utils::testing::node::JormungandrRest;
use serde::{Deserialize, Serialize};
pub const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum VoteTime {
    Blockchain(VoteBlockchainTime),
    Real {
        vote_start_timestamp: NaiveDateTime,
        tally_start_timestamp: NaiveDateTime,
        tally_end_timestamp: NaiveDateTime,
        find_best_match: bool,
    },
}

impl VoteTime {
    pub fn real_from_str<S: Into<String>>(
        vote_start_timestamp: S,
        tally_start_timestamp: S,
        tally_end_timestamp: S,
    ) -> Result<Self> {
        Ok(Self::Real {
            vote_start_timestamp: NaiveDateTime::parse_from_str(
                &vote_start_timestamp.into(),
                FORMAT,
            )?,
            tally_start_timestamp: NaiveDateTime::parse_from_str(
                &tally_start_timestamp.into(),
                FORMAT,
            )?,
            tally_end_timestamp: NaiveDateTime::parse_from_str(
                &tally_end_timestamp.into(),
                FORMAT,
            )?,
            find_best_match: false,
        })
    }

    pub fn real(
        vote_start_timestamp: NaiveDateTime,
        tally_start_timestamp: NaiveDateTime,
        tally_end_timestamp: NaiveDateTime,
    ) -> Self {
        Self::Real {
            vote_start_timestamp,
            tally_start_timestamp,
            tally_end_timestamp,
            find_best_match: false,
        }
    }

    pub fn blockchain(
        vote_start: u32,
        tally_start: u32,
        tally_end: u32,
        slots_per_epoch: u32,
    ) -> Self {
        Self::Blockchain(VoteBlockchainTime {
            vote_start,
            tally_start,
            tally_end,
            slots_per_epoch,
        })
    }
}

impl Default for VoteTime {
    fn default() -> Self {
        Self::Blockchain(VoteBlockchainTime::default())
    }
}

impl Default for VoteBlockchainTime {
    fn default() -> Self {
        Self {
            vote_start: 1,
            tally_start: 2,
            tally_end: 3,
            slots_per_epoch: 30,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct VoteBlockchainTime {
    pub vote_start: u32,
    pub tally_start: u32,
    pub tally_end: u32,
    pub slots_per_epoch: u32,
}

impl VoteBlockchainTime {
    pub fn wait_for_tally_start(self, rest: JormungandrRest) {
        time::wait_for_epoch(self.tally_start, rest);
    }

    pub fn wait_for_tally_end(self, rest: JormungandrRest) {
        time::wait_for_epoch(self.tally_end, rest);
    }
}

impl From<VoteBlockchainTime> for VoteTime {
    fn from(vote_blockchain_time: VoteBlockchainTime) -> Self {
        VoteTime::Blockchain(vote_blockchain_time)
    }
}
