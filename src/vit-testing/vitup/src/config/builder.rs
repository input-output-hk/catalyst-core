pub use crate::builders::ReviewGenerator;
pub use crate::builders::{
    convert_to_blockchain_date, convert_to_human_date, default_next_snapshot_date,
    default_next_vote_date, default_snapshot_date, generate_qr_and_hashes, VitVotePlanDefBuilder,
    WalletExtension,
};
use crate::config::{Config, Initials, VoteTime};
use chain_impl_mockchain::fee::LinearFee;
use chrono::naive::NaiveDateTime;
use jormungandr_lib::interfaces::CommitteeIdDef;
use jormungandr_lib::interfaces::ConsensusLeaderId;
pub use jormungandr_lib::interfaces::Initial;

use crate::config::vote_time::FORMAT;

#[derive(Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn fees(mut self, fees: LinearFee) {
        self.config.blockchain.linear_fees = fees;
    }

    pub fn set_external_committees(mut self, external_committees: Vec<CommitteeIdDef>) {
        self.config.blockchain.committees = external_committees;
    }

    pub fn initials(mut self, initials: Initials) -> Self {
        self.config.initials = initials;
        self
    }

    pub fn block_content_max_size(mut self, block_content_max_size: u32) -> Self {
        self.config.blockchain.block_content_max_size = block_content_max_size;
        self
    }

    pub fn initials_count(self, initials_count: usize, pin: &str) -> Self {
        self.initials(Initials::new_above_threshold(
            initials_count,
            &pin.to_string(),
        ))
    }

    pub fn extend_initials(mut self, initials: Vec<Initial>) -> Self {
        self.config.initials.extend_from_external(initials);
        self
    }

    pub fn slot_duration_in_seconds(mut self, slot_duration: u8) -> Self {
        self.config.blockchain.slot_duration = slot_duration;
        self
    }
    pub fn vote_timing(mut self, vote_timing: VoteTime) -> Self {
        self.config.vote_plan.vote_time = vote_timing;
        self
    }

    pub fn version(mut self, version: String) -> Self {
        self.config.version = version;
        self
    }

    pub fn proposals_count(mut self, proposals_count: u32) -> Self {
        self.config.data.proposals = proposals_count;
        self
    }

    pub fn challenges_count(mut self, challenges_count: usize) -> Self {
        self.config.data.challenges = challenges_count;
        self
    }

    pub fn voting_power(mut self, voting_power: u64) -> Self {
        self.config.data.voting_power = voting_power;
        self
    }

    pub fn consensus_leaders_ids(mut self, consensus_leaders_ids: Vec<ConsensusLeaderId>) -> Self {
        self.config.blockchain.consensus_leader_ids = consensus_leaders_ids;
        self
    }

    pub fn next_vote_timestamp(mut self, next_vote_start_time: NaiveDateTime) -> Self {
        self.config.data.next_vote_start_time = next_vote_start_time;
        self
    }

    pub fn next_vote_timestamp_from_string_or_default(
        self,
        next_vote_timestamp: Option<String>,
        default: NaiveDateTime,
    ) -> Self {
        if let Some(next_vote_timestamp) = next_vote_timestamp {
            self.next_vote_timestamp_from_string(next_vote_timestamp)
        } else {
            self.next_vote_timestamp(default)
        }
    }

    pub fn next_vote_timestamp_from_string(self, next_vote_timestamp: String) -> Self {
        self.next_vote_timestamp(
            NaiveDateTime::parse_from_str(&next_vote_timestamp, FORMAT).unwrap(),
        )
    }

    pub fn snapshot_timestamp_from_string_or_default(
        self,
        snapshot_timestamp: Option<String>,
        default: NaiveDateTime,
    ) -> Self {
        if let Some(snapshot_timestamp) = snapshot_timestamp {
            self.snapshot_timestamp_from_string(snapshot_timestamp)
        } else {
            self.snapshot_timestamp(default)
        }
    }

    pub fn snapshot_timestamp(mut self, snapshot_time: NaiveDateTime) -> Self {
        self.config.data.snapshot_time = snapshot_time;
        self
    }

    pub fn snapshot_timestamp_from_string(self, snapshot_timestamp: String) -> Self {
        self.snapshot_timestamp(NaiveDateTime::parse_from_str(&snapshot_timestamp, FORMAT).unwrap())
    }

    pub fn fund_id(mut self, id: i32) -> Self {
        self.config.data.fund_id = id;
        self
    }

    pub fn private(mut self, private: bool) -> Self {
        self.config.vote_plan.private = private;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}
