use crate::{
    types::vit_ss::fund::{Fund, FundNextInfo, FundStageDates, FundWithNext},
    Error, EventDB,
};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};

#[async_trait]
pub trait VitSSFundQueries: Sync + Send + 'static {
    async fn get_fund(&self) -> Result<FundWithNext, Error>;
}

impl EventDB {
    const FUND_QUERY: &'static str = "
    SELECT
    this_fund.row_id AS id,
    this_fund.name AS fund_name,
    this_fund.description AS fund_goal,

    this_fund.registration_snapshot_time AS registration_snapshot_time,

    this_fund.voting_power_threshold AS voting_power_threshold,

    this_fund.start_time AS fund_start_time,
    this_fund.end_time AS fund_end_time,

    this_fund.insight_sharing_start AS insight_sharing_start,
    this_fund.proposal_submission_start AS proposal_submission_start,
    this_fund.refine_proposals_start AS refine_proposals_start,
    this_fund.finalize_proposals_start AS finalize_proposals_start,
    this_fund.proposal_assessment_start AS proposal_assessment_start,
    this_fund.assessment_qa_start AS assessment_qa_start,
    this_fund.snapshot_start AS snapshot_start,
    this_fund.voting_start AS voting_start,
    this_fund.voting_end AS voting_end,
    this_fund.tallying_end AS tallying_end,

    (this_fund.extra->'url'->'results') #>> '{}' AS results_url,
    (this_fund.extra->'url'->'survey') #>> '{}' AS survey_url,

    next_fund.row_id AS next_id,

    next_fund.start_time AS next_fund_start_time,
    next_fund.registration_snapshot_time AS next_registration_snapshot_time,

    next_fund.name AS next_fund_name,

    next_fund.insight_sharing_start AS next_insight_sharing_start,
    next_fund.proposal_submission_start AS next_proposal_submission_start,
    next_fund.refine_proposals_start AS next_refine_proposals_start,
    next_fund.finalize_proposals_start AS next_finalize_proposals_start,
    next_fund.proposal_assessment_start AS next_proposal_assessment_start,
    next_fund.assessment_qa_start AS next_assessment_qa_start,
    next_fund.snapshot_start AS next_snapshot_start,
    next_fund.voting_start AS next_voting_start,
    next_fund.voting_end AS next_voting_end,
    next_fund.tallying_end AS next_tallying_end
    
    FROM event this_fund
    LEFT JOIN event next_fund ON next_fund.row_id = this_fund.row_id + 1
    WHERE this_fund.end_time > CURRENT_TIMESTAMP AT TIME ZONE 'UTC' AND this_fund.start_time < CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
    ORDER BY this_fund.row_id DESC
    LIMIT 1;";
}

#[async_trait]
impl VitSSFundQueries for EventDB {
    async fn get_fund(&self) -> Result<FundWithNext, Error> {
        let conn = self.pool.get().await?;

        let rows = conn.query(Self::FUND_QUERY, &[]).await?;
        let row = rows
            .get(0)
            .ok_or_else(|| Error::NotFound("can not find fund value".to_string()))?;

        let fund = Fund {
            id: row.try_get("id")?,
            fund_name: row.try_get("fund_name")?,
            fund_goal: row.try_get("fund_goal")?,
            voting_power_threshold: row.try_get("voting_power_threshold")?,
            fund_start_time: row
                .try_get::<_, NaiveDateTime>("fund_start_time")?
                .and_local_timezone(Utc)
                .unwrap(),
            fund_end_time: row
                .try_get::<_, NaiveDateTime>("fund_end_time")?
                .and_local_timezone(Utc)
                .unwrap(),
            next_fund_start_time: row
                .try_get::<_, Option<NaiveDateTime>>("next_fund_start_time")?
                .unwrap_or_default()
                .and_local_timezone(Utc)
                .unwrap(),
            registration_snapshot_time: row
                .try_get::<_, Option<NaiveDateTime>>("registration_snapshot_time")?
                .unwrap_or_default()
                .and_local_timezone(Utc)
                .unwrap(),
            next_registration_snapshot_time: row
                .try_get::<_, Option<NaiveDateTime>>("next_registration_snapshot_time")?
                .unwrap_or_default()
                .and_local_timezone(Utc)
                .unwrap(),
            chain_vote_plans: vec![],
            challenges: vec![],
            stage_dates: FundStageDates {
                insight_sharing_start: row
                    .try_get::<_, Option<NaiveDateTime>>("insight_sharing_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                proposal_submission_start: row
                    .try_get::<_, Option<NaiveDateTime>>("proposal_submission_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                refine_proposals_start: row
                    .try_get::<_, Option<NaiveDateTime>>("refine_proposals_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                finalize_proposals_start: row
                    .try_get::<_, Option<NaiveDateTime>>("finalize_proposals_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                proposal_assessment_start: row
                    .try_get::<_, Option<NaiveDateTime>>("proposal_assessment_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                assessment_qa_start: row
                    .try_get::<_, Option<NaiveDateTime>>("assessment_qa_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                snapshot_start: row
                    .try_get::<_, Option<NaiveDateTime>>("snapshot_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                voting_start: row
                    .try_get::<_, Option<NaiveDateTime>>("voting_start")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                voting_end: row
                    .try_get::<_, Option<NaiveDateTime>>("voting_end")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
                tallying_end: row
                    .try_get::<_, Option<NaiveDateTime>>("tallying_end")?
                    .unwrap_or_default()
                    .and_local_timezone(Utc)
                    .unwrap(),
            },
            goals: vec![],
            results_url: row
                .try_get::<_, Option<String>>("results_url")?
                .unwrap_or_default(),
            survey_url: row
                .try_get::<_, Option<String>>("survey_url")?
                .unwrap_or_default(),
            groups: vec![],
        };

        let next = match row.try_get::<_, Option<i32>>("next_id")? {
            Some(id) => Some(FundNextInfo {
                id,
                fund_name: row.try_get("next_fund_name")?,
                stage_dates: FundStageDates {
                    insight_sharing_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_insight_sharing_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    proposal_submission_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_proposal_submission_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    refine_proposals_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_refine_proposals_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    finalize_proposals_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_finalize_proposals_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    proposal_assessment_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_proposal_assessment_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    assessment_qa_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_assessment_qa_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    snapshot_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_snapshot_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    voting_start: row
                        .try_get::<_, Option<NaiveDateTime>>("next_voting_start")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    voting_end: row
                        .try_get::<_, Option<NaiveDateTime>>("next_voting_end")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                    tallying_end: row
                        .try_get::<_, Option<NaiveDateTime>>("next_tallying_end")?
                        .unwrap_or_default()
                        .and_local_timezone(Utc)
                        .unwrap(),
                },
            }),
            None => None,
        };

        Ok(FundWithNext { fund, next })
    }
}

/// Need to setup and run a test event db instance
/// To do it you can use the following commands:
/// Prepare docker images
/// ```
/// earthly ./containers/event-db-migrations+docker --data=test
/// ```
/// Run event-db container
/// ```
/// docker-compose -f src/event-db/docker-compose.yml up migrations
/// ```
/// Also need establish `EVENT_DB_URL` env variable with the following value
/// ```
/// EVENT_DB_URL="postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
/// ```
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {
    use super::*;
    use crate::establish_connection;
    use chrono::{DateTime, NaiveDate, NaiveTime};

    #[tokio::test]
    async fn get_fund_test() {
        let event_db = establish_connection(None).await.unwrap();

        let fund = event_db.get_fund().await.unwrap();
        assert_eq!(
            fund,
            FundWithNext {
                fund: Fund {
                    id: 4,
                    fund_name: "Test Fund 4".to_string(),
                    fund_goal: "Test Fund 4 description".to_string(),
                    voting_power_threshold: 1,
                    fund_start_time: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    fund_end_time: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    next_fund_start_time: DateTime::<Utc>::from_utc(NaiveDateTime::default(), Utc),
                    registration_snapshot_time: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    next_registration_snapshot_time: DateTime::<Utc>::from_utc(
                        NaiveDateTime::default(),
                        Utc
                    ),
                    chain_vote_plans: vec![],
                    challenges: vec![],
                    stage_dates: FundStageDates {
                        insight_sharing_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        proposal_submission_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        refine_proposals_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        finalize_proposals_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        proposal_assessment_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        assessment_qa_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        snapshot_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        voting_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        voting_end: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                        tallying_end: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        ),
                    },
                    goals: vec![],
                    groups: vec![],
                    survey_url: "".to_string(),
                    results_url: "".to_string(),
                },
                next: Some(FundNextInfo {
                    id: 5,
                    fund_name: "Test Fund 5".to_string(),
                    stage_dates: FundStageDates {
                        insight_sharing_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::default(),
                            Utc
                        ),
                        proposal_submission_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::default(),
                            Utc
                        ),
                        refine_proposals_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::default(),
                            Utc
                        ),
                        finalize_proposals_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::default(),
                            Utc
                        ),
                        proposal_assessment_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::default(),
                            Utc
                        ),
                        assessment_qa_start: DateTime::<Utc>::from_utc(
                            NaiveDateTime::default(),
                            Utc
                        ),
                        snapshot_start: DateTime::<Utc>::from_utc(NaiveDateTime::default(), Utc),
                        voting_start: DateTime::<Utc>::from_utc(NaiveDateTime::default(), Utc),
                        voting_end: DateTime::<Utc>::from_utc(NaiveDateTime::default(), Utc),
                        tallying_end: DateTime::<Utc>::from_utc(NaiveDateTime::default(), Utc),
                    },
                })
            }
        )
    }
}
