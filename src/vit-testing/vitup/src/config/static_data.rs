use crate::config::convert_to_human_date;
use crate::config::Config;
use rand::Rng;
use serde::{Deserialize, Serialize};
use time::{ext::NumericalDuration, OffsetDateTime};
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;
use vit_servicing_station_tests::common::data::ValidVotePlanDates;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaticData {
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_vote_options_from_string"
    )]
    pub options: VoteOptions,
    pub proposals: u32,
    pub challenges: usize,
    pub reviews: usize,
    pub voting_power: u64,
    pub fund_name: String,
    #[serde(default = "default_fund_id")]
    pub fund_id: i32,
    pub dates: Dates,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dates {
    #[serde(with = "time::serde::rfc3339")]
    pub next_vote_start_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub snapshot_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub next_snapshot_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub insight_sharing_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub proposal_submission_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub refine_proposals_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub finalize_proposals_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub proposal_assessment_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub assessment_qa_start: OffsetDateTime,
}

impl Dates {
    pub fn as_valid_date(&self, config: &Config) -> ValidVotePlanDates {
        let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
            convert_to_human_date(config);

        ValidVotePlanDates {
            next_fund_start_time: self.next_vote_start_time.unix_timestamp(),
            registration_snapshot_time: self.snapshot_time.unix_timestamp(),
            next_registration_snapshot_time: self.next_snapshot_time.unix_timestamp(),
            insight_sharing_start: self.insight_sharing_start.unix_timestamp(),
            proposal_submission_start: self.proposal_submission_start.unix_timestamp(),
            refine_proposals_start: self.refine_proposals_start.unix_timestamp(),
            finalize_proposals_start: self.finalize_proposals_start.unix_timestamp(),
            proposal_assessment_start: self.proposal_assessment_start.unix_timestamp(),
            assessment_qa_start: self.assessment_qa_start.unix_timestamp(),
            voting_start: vote_start_timestamp.unix_timestamp(),
            voting_tally_end: tally_end_timestamp.unix_timestamp(),
            voting_tally_start: tally_start_timestamp.unix_timestamp(),
        }
    }
}

impl Default for Dates {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            next_vote_start_time: now + 30.days(),
            snapshot_time: now - 3.hours(),
            next_snapshot_time: now + 20.days(),
            insight_sharing_start: now - 10.days(),
            proposal_submission_start: now - 9.days(),
            refine_proposals_start: now - 8.days(),
            finalize_proposals_start: now - 7.days(),
            proposal_assessment_start: now - 6.days(),
            assessment_qa_start: now - 5.days(),
        }
    }
}

impl Default for StaticData {
    fn default() -> Self {
        Self {
            proposals: 100,
            challenges: 9,
            reviews: 3,
            voting_power: 8000,
            options: VoteOptions::parse_coma_separated_value("yes,no"),
            fund_name: "fund_3".to_owned(),
            fund_id: default_fund_id(),
            dates: Default::default(),
        }
    }
}

fn default_fund_id() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..10_000) + 1
}
