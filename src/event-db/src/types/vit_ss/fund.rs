use super::{challenge::Challenge, goal::Goal, group::Group, vote_plan::Voteplan};
use crate::types::utils::serialize_datetime_as_rfc3339;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct FundStageDates {
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub insight_sharing_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub proposal_submission_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub refine_proposals_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub finalize_proposals_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub proposal_assessment_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub assessment_qa_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub snapshot_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub voting_start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub voting_end: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub tallying_end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Fund {
    pub id: i32,
    pub fund_name: String,
    pub fund_goal: String,
    pub voting_power_threshold: i64,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub fund_start_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub fund_end_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub next_fund_start_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub registration_snapshot_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub next_registration_snapshot_time: DateTime<Utc>,
    pub chain_vote_plans: Vec<Voteplan>,
    pub challenges: Vec<Challenge>,
    #[serde(flatten)]
    pub stage_dates: FundStageDates,
    pub goals: Vec<Goal>,
    pub results_url: String,
    pub survey_url: String,
    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct FundNextInfo {
    pub id: i32,
    pub fund_name: String,
    #[serde(flatten)]
    pub stage_dates: FundStageDates,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct FundWithNext {
    #[serde(flatten)]
    pub fund: Fund,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<FundNextInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;

    #[test]
    fn fund_json_test() {
        let fund = Fund {
            id: 1,
            fund_name: "fund_name 1".to_string(),
            fund_goal: "fund_goal 1".to_string(),
            voting_power_threshold: 1,
            fund_start_time: DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            ),
            fund_end_time: DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            ),
            next_fund_start_time: DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            ),
            registration_snapshot_time: DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            ),
            next_registration_snapshot_time: DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            ),
            chain_vote_plans: vec![],
            challenges: vec![],
            stage_dates: FundStageDates {
                insight_sharing_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                proposal_submission_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                refine_proposals_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                finalize_proposals_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                proposal_assessment_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                assessment_qa_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                snapshot_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                voting_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                voting_end: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                tallying_end: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
            },
            goals: vec![],
            results_url: "results_url 1".to_string(),
            survey_url: "survey_url 1".to_string(),
            groups: vec![],
        };

        let json = serde_json::to_value(&fund).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "fund_name": "fund_name 1",
                    "fund_goal": "fund_goal 1",
                    "voting_power_threshold": 1,
                    "fund_start_time": "1970-01-01T00:00:00+00:00",
                    "fund_end_time": "1970-01-01T00:00:00+00:00",
                    "next_fund_start_time": "1970-01-01T00:00:00+00:00",
                    "registration_snapshot_time": "1970-01-01T00:00:00+00:00",
                    "next_registration_snapshot_time": "1970-01-01T00:00:00+00:00",
                    "chain_vote_plans": [],
                    "challenges": [],
                    "insight_sharing_start": "1970-01-01T00:00:00+00:00",
                    "proposal_submission_start": "1970-01-01T00:00:00+00:00",
                    "refine_proposals_start": "1970-01-01T00:00:00+00:00",
                    "finalize_proposals_start": "1970-01-01T00:00:00+00:00",
                    "proposal_assessment_start": "1970-01-01T00:00:00+00:00",
                    "assessment_qa_start": "1970-01-01T00:00:00+00:00",
                    "snapshot_start": "1970-01-01T00:00:00+00:00",
                    "voting_start": "1970-01-01T00:00:00+00:00",
                    "voting_end": "1970-01-01T00:00:00+00:00",
                    "tallying_end": "1970-01-01T00:00:00+00:00",
                    "goals": [],
                    "results_url": "results_url 1",
                    "survey_url": "survey_url 1",
                    "groups": [],
                }
            )
        )
    }

    #[test]
    fn fund_next_info_json_test() {
        let fund_next_info = FundNextInfo {
            id: 1,
            fund_name: "fund_name 1".to_string(),
            stage_dates: FundStageDates {
                insight_sharing_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                proposal_submission_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                refine_proposals_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                finalize_proposals_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                proposal_assessment_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                assessment_qa_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                snapshot_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                voting_start: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                voting_end: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                tallying_end: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
            },
        };

        let json = serde_json::to_value(&fund_next_info).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "fund_name": "fund_name 1",
                    "insight_sharing_start": "1970-01-01T00:00:00+00:00",
                    "proposal_submission_start": "1970-01-01T00:00:00+00:00",
                    "refine_proposals_start": "1970-01-01T00:00:00+00:00",
                    "finalize_proposals_start": "1970-01-01T00:00:00+00:00",
                    "proposal_assessment_start": "1970-01-01T00:00:00+00:00",
                    "assessment_qa_start": "1970-01-01T00:00:00+00:00",
                    "snapshot_start": "1970-01-01T00:00:00+00:00",
                    "voting_start": "1970-01-01T00:00:00+00:00",
                    "voting_end": "1970-01-01T00:00:00+00:00",
                    "tallying_end": "1970-01-01T00:00:00+00:00",
                }
            )
        )
    }
}
