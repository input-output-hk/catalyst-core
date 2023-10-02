use crate::common::data::generator::voting::parameters::FundStageDates;
use time::{ext::NumericalDuration, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct FundDates {
    pub voting_start: i64,
    pub voting_tally_start: i64,
    pub voting_tally_end: i64,
    pub next_fund_start_time: i64,
    pub registration_snapshot_time: i64,
    pub next_registration_snapshot_time: i64,
    pub insight_sharing_start: i64,
    pub proposal_submission_start: i64,
    pub refine_proposals_start: i64,
    pub finalize_proposals_start: i64,
    pub proposal_assessment_start: i64,
    pub assessment_qa_start: i64,
}

impl Default for FundDates {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            proposal_submission_start: as_timestamp(now - 10.days()),
            insight_sharing_start: as_timestamp(now - 9.days()),
            refine_proposals_start: as_timestamp(now - 8.days()),
            finalize_proposals_start: as_timestamp(now - 7.days()),
            proposal_assessment_start: as_timestamp(now - 6.days()),
            assessment_qa_start: as_timestamp(now - 5.days()),
            registration_snapshot_time: as_timestamp(now - 4.days()),
            voting_start: as_timestamp(now + 1.days()),
            voting_tally_start: as_timestamp(now + 2.days()),
            voting_tally_end: as_timestamp(now + 3.days()),
            next_registration_snapshot_time: as_timestamp(now + 7.days()),
            next_fund_start_time: as_timestamp(now + 10.days()),
        }
    }
}

fn as_timestamp(date: OffsetDateTime) -> i64 {
    date.unix_timestamp()
}

#[allow(clippy::from_over_into)]
impl Into<FundStageDates> for FundDates {
    fn into(self) -> FundStageDates {
        FundStageDates {
            insight_sharing_start: self.insight_sharing_start,
            proposal_submission_start: self.proposal_submission_start,
            refine_proposals_start: self.refine_proposals_start,
            finalize_proposals_start: self.finalize_proposals_start,
            proposal_assessment_start: self.proposal_assessment_start,
            assessment_qa_start: self.assessment_qa_start,
            snapshot_start: self.registration_snapshot_time,
            voting_start: self.voting_start,
            voting_end: self.voting_tally_start,
            tallying_end: self.voting_tally_end,
        }
    }
}
