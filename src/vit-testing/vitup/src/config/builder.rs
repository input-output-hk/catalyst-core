pub use crate::builders::convert_to_human_date;
use crate::config::Block0Initials;
use crate::config::{date_format, Initials};
use crate::config::{Config, VoteTime};
use chain_addr::Discrimination;
use chain_impl_mockchain::fee::LinearFee;
use jormungandr_lib::interfaces::CommitteeIdDef;
use jormungandr_lib::interfaces::ConsensusLeaderId;
use mainnet_lib::Initials as SnapshotInitials;
use snapshot_lib::VoterHIR;
use time::OffsetDateTime;

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

    pub fn block0_initials(mut self, initials: Block0Initials) -> Self {
        self.config.initials.block0 = initials;
        self
    }

    pub fn snapshot_initials(mut self, initials: SnapshotInitials) -> Self {
        self.config.initials.snapshot = Some(initials);
        self
    }

    pub fn block_content_max_size(mut self, block_content_max_size: u32) -> Self {
        self.config.blockchain.block_content_max_size = block_content_max_size;
        self
    }

    pub fn block0_initials_count(self, initials_count: usize, pin: &str) -> Self {
        self.block0_initials(Block0Initials::new_above_threshold(initials_count, pin))
    }

    pub fn extend_block0_initials(
        mut self,
        initials: Vec<VoterHIR>,
        discrimination: Discrimination,
    ) -> Self {
        self.config
            .initials
            .block0
            .extend_from_external(initials, discrimination);
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
        self.config.service.version = version;
        self
    }

    pub fn proposals_count(mut self, proposals_count: u32) -> Self {
        self.config.data.current_fund.proposals = proposals_count;
        self
    }

    pub fn challenges_count(mut self, challenges_count: usize) -> Self {
        self.config.data.current_fund.challenges = challenges_count;
        self
    }

    pub fn reviews_count(mut self, reviews_count: usize) -> Self {
        self.config.data.current_fund.reviews = reviews_count;
        self
    }

    pub fn voting_power(mut self, voting_power: u64) -> Self {
        self.config.data.current_fund.voting_power = voting_power;
        self
    }

    pub fn consensus_leaders_ids(mut self, consensus_leaders_ids: Vec<ConsensusLeaderId>) -> Self {
        self.config.blockchain.consensus_leader_ids = consensus_leaders_ids;
        self
    }

    pub fn next_vote_timestamp(mut self, next_vote_start_time: OffsetDateTime) -> Self {
        self.config.data.current_fund.dates.next_vote_start_time = next_vote_start_time;
        self
    }

    pub fn next_vote_timestamp_from_string_if_some(
        self,
        next_vote_timestamp: Option<String>,
    ) -> Self {
        if let Some(next_vote_timestamp) = next_vote_timestamp {
            self.next_vote_timestamp_from_string(next_vote_timestamp)
        } else {
            self
        }
    }

    pub fn next_vote_timestamp_from_string(self, next_vote_timestamp: String) -> Self {
        self.next_vote_timestamp(
            OffsetDateTime::parse(&next_vote_timestamp, &date_format()).unwrap(),
        )
    }

    pub fn snapshot_timestamp_from_string_if_some(
        self,
        snapshot_timestamp: Option<String>,
    ) -> Self {
        if let Some(snapshot_timestamp) = snapshot_timestamp {
            self.snapshot_timestamp_from_string(snapshot_timestamp)
        } else {
            self
        }
    }

    pub fn snapshot_timestamp(mut self, snapshot_time: OffsetDateTime) -> Self {
        self.config.data.current_fund.dates.snapshot_time = snapshot_time;
        self
    }

    pub fn snapshot_timestamp_from_string(self, snapshot_timestamp: String) -> Self {
        self.snapshot_timestamp(OffsetDateTime::parse(&snapshot_timestamp, &date_format()).unwrap())
    }

    pub fn fund_id(mut self, id: i32) -> Self {
        self.config.data.current_fund.fund_info.fund_id = id;
        self
    }

    pub fn private(mut self, private: bool) -> Self {
        self.config.vote_plan.private = private;
        self
    }

    pub fn use_https(mut self) -> Self {
        self.config.service.https = true;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}
