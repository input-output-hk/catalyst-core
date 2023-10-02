use crate::db::{
    models::{challenges::Challenge, goals::Goal, voteplans::Voteplan},
    schema::funds,
    Db,
};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Fund {
    #[serde(default = "Default::default")]
    pub id: i32,
    #[serde(alias = "fundName")]
    pub fund_name: String,
    #[serde(alias = "fundGoal")]
    pub fund_goal: String,
    #[serde(alias = "votingPowerThreshold")]
    pub voting_power_threshold: i64,
    #[serde(alias = "fundStartTime")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub fund_start_time: i64,
    #[serde(alias = "fundEndTime")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub fund_end_time: i64,
    #[serde(alias = "nextFundStartTime")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub next_fund_start_time: i64,
    #[serde(alias = "registrationSnapshotTime")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub registration_snapshot_time: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub next_registration_snapshot_time: i64,
    #[serde(alias = "chainVotePlans", default = "Vec::new")]
    pub chain_vote_plans: Vec<Voteplan>,
    #[serde(default = "Vec::new")]
    pub challenges: Vec<Challenge>,
    #[serde(alias = "stageDates", flatten)]
    pub stage_dates: FundStageDates,
    #[serde(default = "Vec::new")]
    pub goals: Vec<Goal>,
    #[serde(alias = "resultsUrl")]
    pub results_url: String,
    #[serde(alias = "surveyUrl")]
    pub survey_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FundStageDates {
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub insight_sharing_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub proposal_submission_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub refine_proposals_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub finalize_proposals_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub proposal_assessment_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub assessment_qa_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub snapshot_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub voting_start: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub voting_end: i64,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub tallying_end: i64,
}

#[derive(Serialize)]
struct FundWithLegacyFields {
    id: i32,
    fund_name: String,
    fund_goal: String,
    voting_power_threshold: i64,
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    fund_start_time: i64,
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    fund_end_time: i64,
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    next_fund_start_time: i64,
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    registration_snapshot_time: i64,
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    next_registration_snapshot_time: i64,
    chain_vote_plans: Vec<Voteplan>,
    challenges: Vec<Challenge>,
}

impl From<Fund> for FundWithLegacyFields {
    fn from(fund: Fund) -> Self {
        FundWithLegacyFields {
            id: fund.id,
            fund_name: fund.fund_name,
            fund_goal: fund.fund_goal,
            voting_power_threshold: fund.voting_power_threshold,
            fund_start_time: fund.fund_start_time,
            fund_end_time: fund.fund_end_time,
            next_fund_start_time: fund.next_fund_start_time,
            registration_snapshot_time: fund.registration_snapshot_time,
            next_registration_snapshot_time: fund.next_registration_snapshot_time,
            chain_vote_plans: fund.chain_vote_plans,
            challenges: fund.challenges,
        }
    }
}

impl Queryable<funds::SqlType, Db> for Fund {
    type Row = (
        // 0 -> id
        i32,
        // 1 -> fund_name
        String,
        // 2 -> fund_goal
        String,
        // 3 -> registration_snapshot_time
        i64,
        // 4 -> next_registration_snapshot_time
        i64,
        // 5 -> voting_power_threshold
        i64,
        // 6 -> fund_start_time
        i64,
        // 7 -> fund_end_time
        i64,
        // 8 -> next_fund_start_time
        i64,
        // insight_sharing_start
        i64,
        // proposal_submission_start
        i64,
        // refine_proposals_start
        i64,
        // finalize_proposals_start
        i64,
        // proposal_assessment_start
        i64,
        // assessment_qa_start
        i64,
        // snapshot_start
        i64,
        // voting_start
        i64,
        // voting_end
        i64,
        // tallying_end
        i64,
        // results_url
        String,
        // survey_url
        String,
    );

    fn build(row: Self::Row) -> Self {
        Fund {
            id: row.0,
            fund_name: row.1,
            fund_goal: row.2,
            registration_snapshot_time: row.3,
            next_registration_snapshot_time: row.4,
            voting_power_threshold: row.5,
            fund_start_time: row.6,
            fund_end_time: row.7,
            next_fund_start_time: row.8,
            chain_vote_plans: vec![],
            challenges: vec![],
            stage_dates: FundStageDates {
                insight_sharing_start: row.9,
                proposal_submission_start: row.10,
                refine_proposals_start: row.11,
                finalize_proposals_start: row.12,
                proposal_assessment_start: row.13,
                assessment_qa_start: row.14,
                snapshot_start: row.15,
                voting_start: row.16,
                voting_end: row.17,
                tallying_end: row.18,
            },
            goals: vec![],
            results_url: row.19,
            survey_url: row.20,
        }
    }
}

// This warning is disabled here. Values is only referenced as a type here. It should be ok not to
// split the types definitions.
#[allow(clippy::type_complexity)]
impl Insertable<funds::table> for Fund {
    type Values = (
        Option<diesel::dsl::Eq<funds::id, i32>>,
        diesel::dsl::Eq<funds::fund_name, String>,
        diesel::dsl::Eq<funds::fund_goal, String>,
        diesel::dsl::Eq<funds::registration_snapshot_time, i64>,
        diesel::dsl::Eq<funds::next_registration_snapshot_time, i64>,
        diesel::dsl::Eq<funds::voting_power_threshold, i64>,
        diesel::dsl::Eq<funds::fund_start_time, i64>,
        diesel::dsl::Eq<funds::fund_end_time, i64>,
        diesel::dsl::Eq<funds::next_fund_start_time, i64>,
        diesel::dsl::Eq<funds::insight_sharing_start, i64>,
        diesel::dsl::Eq<funds::proposal_submission_start, i64>,
        diesel::dsl::Eq<funds::refine_proposals_start, i64>,
        diesel::dsl::Eq<funds::finalize_proposals_start, i64>,
        diesel::dsl::Eq<funds::proposal_assessment_start, i64>,
        diesel::dsl::Eq<funds::assessment_qa_start, i64>,
        diesel::dsl::Eq<funds::snapshot_start, i64>,
        diesel::dsl::Eq<funds::voting_start, i64>,
        diesel::dsl::Eq<funds::voting_end, i64>,
        diesel::dsl::Eq<funds::tallying_end, i64>,
        diesel::dsl::Eq<funds::results_url, String>,
        diesel::dsl::Eq<funds::survey_url, String>,
    );

    fn values(self) -> Self::Values {
        let id_item = if self.id == 0 {
            None
        } else {
            Some(funds::id.eq(self.id))
        };
        (
            id_item,
            funds::fund_name.eq(self.fund_name),
            funds::fund_goal.eq(self.fund_goal),
            funds::registration_snapshot_time.eq(self.registration_snapshot_time),
            funds::next_registration_snapshot_time.eq(self.next_registration_snapshot_time),
            funds::voting_power_threshold.eq(self.voting_power_threshold),
            funds::fund_start_time.eq(self.fund_start_time),
            funds::fund_end_time.eq(self.fund_end_time),
            funds::next_fund_start_time.eq(self.next_fund_start_time),
            funds::insight_sharing_start.eq(self.stage_dates.insight_sharing_start),
            funds::proposal_submission_start.eq(self.stage_dates.proposal_submission_start),
            funds::refine_proposals_start.eq(self.stage_dates.refine_proposals_start),
            funds::finalize_proposals_start.eq(self.stage_dates.finalize_proposals_start),
            funds::proposal_assessment_start.eq(self.stage_dates.proposal_assessment_start),
            funds::assessment_qa_start.eq(self.stage_dates.assessment_qa_start),
            funds::snapshot_start.eq(self.stage_dates.snapshot_start),
            funds::voting_start.eq(self.stage_dates.voting_start),
            funds::voting_end.eq(self.stage_dates.voting_end),
            funds::tallying_end.eq(self.stage_dates.tallying_end),
            funds::results_url.eq(self.results_url),
            funds::survey_url.eq(self.survey_url),
        )
    }
}

#[cfg(test)]
pub mod test {
    use crate::db::{
        models::{
            challenges::test as challenges_testing,
            funds::{Fund, FundStageDates},
            goals::{Goal, InsertGoal},
            voteplans::test as voteplans_testing,
        },
        schema::{funds, goals},
        DbConnectionPool,
    };

    use diesel::{Insertable, RunQueryDsl};
    use time::{Duration, OffsetDateTime};

    pub fn get_test_fund(fund_id: Option<i32>) -> Fund {
        const FUND_ID: i32 = 42;
        let fund_id = fund_id.unwrap_or(FUND_ID);

        Fund {
            id: fund_id,
            fund_name: "hey oh let's go".to_string(),
            fund_goal: "test this endpoint".to_string(),
            registration_snapshot_time: (OffsetDateTime::now_utc() + Duration::days(3))
                .unix_timestamp(),
            next_registration_snapshot_time: (OffsetDateTime::now_utc() + Duration::days(30))
                .unix_timestamp(),
            voting_power_threshold: 100,
            fund_start_time: OffsetDateTime::now_utc().unix_timestamp(),
            fund_end_time: OffsetDateTime::now_utc().unix_timestamp(),
            next_fund_start_time: OffsetDateTime::now_utc().unix_timestamp(),
            chain_vote_plans: vec![voteplans_testing::get_test_voteplan_with_fund_id(fund_id)],
            challenges: vec![challenges_testing::get_test_challenge_with_fund_id(fund_id)],
            stage_dates: FundStageDates {
                insight_sharing_start: OffsetDateTime::now_utc().unix_timestamp(),
                proposal_submission_start: OffsetDateTime::now_utc().unix_timestamp(),
                refine_proposals_start: OffsetDateTime::now_utc().unix_timestamp(),
                finalize_proposals_start: OffsetDateTime::now_utc().unix_timestamp(),
                proposal_assessment_start: OffsetDateTime::now_utc().unix_timestamp(),
                assessment_qa_start: OffsetDateTime::now_utc().unix_timestamp(),
                snapshot_start: OffsetDateTime::now_utc().unix_timestamp(),
                voting_start: OffsetDateTime::now_utc().unix_timestamp(),
                voting_end: OffsetDateTime::now_utc().unix_timestamp(),
                tallying_end: OffsetDateTime::now_utc().unix_timestamp(),
            },
            goals: vec![Goal {
                id: 1,
                goal_name: "goal1".into(),
                fund_id,
            }],
            results_url: format!("http://localhost/fund/{FUND_ID}/results/"),
            survey_url: format!("http://localhost/fund/{FUND_ID}/survey/"),
        }
    }

    pub fn populate_db_with_fund(fund: &Fund, pool: &DbConnectionPool) {
        let values = fund.clone().values();

        // Warning! mind this scope: r2d2 pooled connection behaviour depend of the scope. Looks like
        // if the connection is not out of scope, when giving the reference to the next function
        // call below it creates a wrong connection (where there are not tables even if they were loaded).
        {
            let connection = pool.get().unwrap();
            diesel::insert_into(funds::table)
                .values(values)
                .execute(&connection)
                .unwrap();
        }

        for voteplan in &fund.chain_vote_plans {
            voteplans_testing::populate_db_with_voteplan(voteplan, pool);
        }

        for challenge in &fund.challenges {
            challenges_testing::populate_db_with_challenge(challenge, pool);
        }

        {
            let connection = pool.get().unwrap();
            for goal in &fund.goals {
                diesel::insert_into(goals::table)
                    .values(InsertGoal::from(goal))
                    .execute(&connection)
                    .unwrap();
            }
        }
    }
}
