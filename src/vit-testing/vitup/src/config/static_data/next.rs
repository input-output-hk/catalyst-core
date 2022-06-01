use super::FundInfo;
use serde::{Deserialize, Serialize};
use time::{ext::NumericalDuration, OffsetDateTime};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NextFund {
    #[serde(flatten)]
    pub fund_info: FundInfo,
    pub dates: NextFundDates,
}

impl Default for NextFund {
    fn default() -> Self {
        Self {
            dates: Default::default(),
            fund_info: 10i32.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NextFundDates {
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
    #[serde(with = "time::serde::rfc3339")]
    pub snapshot_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub voting_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub voting_tally_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub voting_tally_end: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub next_snapshot_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub next_vote_start_time: OffsetDateTime,
}

impl Default for NextFundDates {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            insight_sharing_start: now + 5.days(),
            proposal_submission_start: now + 6.days(),
            refine_proposals_start: now + 7.days(),
            finalize_proposals_start: now + 8.days(),
            proposal_assessment_start: now + 9.days(),
            assessment_qa_start: now + 10.days(),
            snapshot_time: now + 20.days(),
            voting_start: now + 30.days(),
            voting_tally_start: now + 31.days(),
            voting_tally_end: now + 32.days(),
            next_snapshot_time: now + 40.days(),
            next_vote_start_time: now + 50.days(),
        }
    }
}
