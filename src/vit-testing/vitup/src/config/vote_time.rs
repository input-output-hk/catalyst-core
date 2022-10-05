use crate::config::date_format;
use crate::Result;
use jormungandr_automation::{jormungandr::JormungandrRest, testing::time::wait_for_epoch};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
pub const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VoteTime {
    Blockchain(VoteBlockchainTime),
    Real {
        #[serde(with = "time::serde::rfc3339")]
        vote_start_timestamp: OffsetDateTime,
        #[serde(with = "time::serde::rfc3339")]
        tally_start_timestamp: OffsetDateTime,
        #[serde(with = "time::serde::rfc3339")]
        tally_end_timestamp: OffsetDateTime,
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
            vote_start_timestamp: OffsetDateTime::parse(
                &vote_start_timestamp.into(),
                &date_format(),
            )?,
            tally_start_timestamp: OffsetDateTime::parse(
                &tally_start_timestamp.into(),
                &date_format(),
            )?,
            tally_end_timestamp: OffsetDateTime::parse(
                &tally_end_timestamp.into(),
                &date_format(),
            )?,
            find_best_match: false,
        })
    }

    pub fn real(
        vote_start_timestamp: OffsetDateTime,
        tally_start_timestamp: OffsetDateTime,
        tally_end_timestamp: OffsetDateTime,
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
    pub fn wait_for_vote_start(self, rest: JormungandrRest) {
        wait_for_epoch(self.vote_start, rest);
    }

    pub fn wait_for_tally_start(self, rest: JormungandrRest) {
        wait_for_epoch(self.tally_start, rest);
    }

    pub fn wait_for_tally_end(self, rest: JormungandrRest) {
        wait_for_epoch(self.tally_end, rest);
    }
}

impl From<VoteBlockchainTime> for VoteTime {
    fn from(vote_blockchain_time: VoteBlockchainTime) -> Self {
        VoteTime::Blockchain(vote_blockchain_time)
    }
}
