use super::FundInfo;
use serde::{Deserialize, Serialize};
use time::{ext::NumericalDuration, OffsetDateTime};
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentFund {
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_vote_options_from_string",
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_vote_options_to_string"
    )]
    pub options: VoteOptions,
    pub proposals: u32,
    pub challenges: usize,
    pub reviews: usize,
    pub voting_power: u64,
    #[serde(flatten)]
    pub fund_info: FundInfo,
    pub dates: CurrentFundDates,
}

impl Default for CurrentFund {
    fn default() -> Self {
        Self {
            proposals: 100,
            challenges: 9,
            reviews: 3,
            voting_power: 8000,
            options: VoteOptions::parse_coma_separated_value("yes,no"),
            dates: Default::default(),
            fund_info: 9i32.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentFundDates {
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

impl Default for CurrentFundDates {
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
