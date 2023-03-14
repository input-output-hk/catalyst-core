use std::collections::BTreeMap;

use diesel::RunQueryDsl;
use thiserror::Error;
use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData,
    challenges::Challenge,
    community_advisors_reviews::AdvisorReview,
    funds::Fund,
    groups::Group,
    proposals::{FullProposalInfo, ProposalChallengeInfo},
};
use vit_servicing_station_lib::db::DbConnection;

#[derive(diesel::QueryableByName)]
struct RowId {
    #[sql_type = "diesel::sql_types::Integer"]
    row_id: i32,
}

pub struct DbInserter<'a> {
    connection: &'a DbConnection,
}

impl<'a> DbInserter<'a> {
    pub fn new(connection: &'a DbConnection) -> Self {
        Self { connection }
    }

    pub fn insert_token(&self, token_data: &ApiTokenData) -> Result<(), DbInserterError> {
        diesel::sql_query("INSERT INTO config (id, id2, id3, value) VALUES ($1, $2, $3, $4)")
            .bind::<diesel::sql_types::VarChar, _>("api_token")
            .bind::<diesel::sql_types::VarChar, _>(base64::encode(token_data.token.as_ref()))
            .bind::<diesel::sql_types::VarChar, _>("")
            .bind::<diesel::sql_types::Jsonb, _>(serde_json::json!({
                "created": token_data.creation_time,
                "expires": token_data.expire_time,
            }))
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)
            .map(|_| ())
    }

    pub fn insert_tokens(&self, tokens_data: &[ApiTokenData]) -> Result<(), DbInserterError> {
        for token_data in tokens_data {
            self.insert_token(token_data)?;
        }
        Ok(())
    }

    pub fn insert_proposals(&self, proposals: &[FullProposalInfo]) -> Result<(), DbInserterError> {
        for proposal in proposals {
            let proposal_id = proposal
                .proposal
                .proposal_id
                .as_str()
                .parse::<i32>()
                .unwrap();

            let mut extra = match &proposal.challenge_info {
                ProposalChallengeInfo::Simple(info) => {
                    vec![("solution", info.proposal_solution.as_str())]
                }
                ProposalChallengeInfo::CommunityChoice(info) => vec![
                    ("brief", info.proposal_brief.as_str()),
                    ("importance", info.proposal_importance.as_str()),
                    ("goal", info.proposal_goal.as_str()),
                    ("metrics", info.proposal_metrics.as_str()),
                ],
            }
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<BTreeMap<String, String>>();

            if let Some(e) = proposal.proposal.extra.as_ref() {
                extra.extend(
                    e.iter()
                        .map(|(a, b)| (a.clone(), b.clone()))
                        .collect::<Vec<_>>(),
                );
            }

            diesel::sql_query(
                r#"
                INSERT INTO proposal (
                    row_id,
                    id,
                    title,
                    summary,
                    category,
                    public_key,
                    funds,
                    url,
                    files_url,
                    impact_score,
                    proposer_name,
                    proposer_contact,
                    proposer_url,
                    proposer_relevant_experience,
                    bb_proposal_id,
                    bb_vote_options,
                    challenge,
                    extra
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, $14, $15, $16, $17, $18
                )
                "#,
            )
            .bind::<diesel::sql_types::Integer, _>(&proposal.proposal.internal_id)
            .bind::<diesel::sql_types::Integer, _>(&proposal_id)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposal_title)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposal_summary)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposal_category.category_name)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposal_public_key)
            .bind::<diesel::sql_types::BigInt, _>(&proposal.proposal.proposal_funds)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposal_url)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposal_files_url)
            .bind::<diesel::sql_types::BigInt, _>(&proposal.proposal.proposal_impact_score)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposer.proposer_name)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposer.proposer_email)
            .bind::<diesel::sql_types::Text, _>(&proposal.proposal.proposer.proposer_url)
            .bind::<diesel::sql_types::Text, _>(
                &proposal.proposal.proposer.proposer_relevant_experience,
            )
            .bind::<diesel::sql_types::Binary, _>(&proposal.proposal.chain_proposal_id)
            .bind::<diesel::sql_types::Text, _>(
                proposal.proposal.chain_vote_options.as_csv_string(),
            )
            .bind::<diesel::sql_types::Integer, _>(&proposal.proposal.challenge_id)
            .bind::<diesel::sql_types::Jsonb, _>(serde_json::to_value(extra).unwrap())
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;

            let res = diesel::sql_query(
                r#"
                SELECT row_id FROM voteplan WHERE id = $1
                "#,
            )
            .bind::<diesel::sql_types::Text, _>(&proposal.voteplan.chain_voteplan_id)
            .get_result::<RowId>(self.connection)
            .map_err(DbInserterError::DieselError)?;

            diesel::sql_query(
                r#"
                INSERT INTO proposal_voteplan (
                    proposal_id,
                    voteplan_id,
                    bb_proposal_index
                ) VALUES ($1, $2, $3)
                "#,
            )
            .bind::<diesel::sql_types::Integer, _>(proposal.proposal.internal_id)
            .bind::<diesel::sql_types::Integer, _>(res.row_id)
            .bind::<diesel::sql_types::BigInt, _>(proposal.voteplan.chain_proposal_index)
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;
        }

        Ok(())
    }

    pub fn insert_funds(&self, funds: &[Fund]) -> Result<(), DbInserterError> {
        for fund in funds {
            let id_item = if fund.id == 0 { None } else { Some(fund.id) };

            diesel::sql_query(
                r#"
                INSERT INTO event (
                    row_id,
                    name,
                    description,
                    registration_snapshot_time,
                    voting_power_threshold,
                    start_time,
                    end_time,
                    insight_sharing_start,
                    proposal_submission_start,
                    refine_proposals_start,
                    finalize_proposals_start,
                    proposal_assessment_start,
                    assessment_qa_start,
                    snapshot_start,
                    voting_start,
                    voting_end,
                    tallying_end,
                    extra,
                    committee_size,
                    committee_threshold
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, $14, $15, $16, $17, $18,
                    $19, $20
                )
                "#,
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(id_item)
            .bind::<diesel::sql_types::Text, _>(&fund.fund_name)
            .bind::<diesel::sql_types::Text, _>(&fund.fund_goal)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.registration_snapshot_time * 1000,
                ),
            )
            .bind::<diesel::sql_types::BigInt, _>(fund.voting_power_threshold)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(fund.fund_start_time * 1000),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(fund.fund_end_time * 1000),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.insight_sharing_start * 1000,
                ),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.proposal_submission_start * 1000,
                ),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.refine_proposals_start * 1000,
                ),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.finalize_proposals_start * 1000,
                ),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.proposal_assessment_start * 1000,
                ),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.assessment_qa_start * 1000,
                ),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.snapshot_start * 1000,
                ),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(fund.stage_dates.voting_start * 1000),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(fund.stage_dates.voting_end * 1000),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamp>, _>(
                chrono::NaiveDateTime::from_timestamp_millis(fund.stage_dates.tallying_end * 1000),
            )
            .bind::<diesel::sql_types::Jsonb, _>(serde_json::json!({
                "url": {
                    "results": fund.results_url,
                    "survey": fund.survey_url,
                }
            }))
            .bind::<diesel::sql_types::Integer, _>(0)
            .bind::<diesel::sql_types::Integer, _>(0)
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;

            self.insert_groups(&fund.groups.iter().cloned().collect::<Vec<_>>())?;

            for voteplan in &fund.chain_vote_plans {
                let res = diesel::sql_query("SELECT row_id FROM voting_group WHERE token_id = $1")
                    .bind::<diesel::sql_types::VarChar, _>(&voteplan.token_identifier)
                    .get_result::<RowId>(self.connection)
                    .map_err(DbInserterError::DieselError)?;

                diesel::sql_query(
                    r#"
                    INSERT INTO voteplan (
                        row_id,
                        id,
                        category,
                        event_id,
                        encryption_key,
                        group_id
                    ) VALUES (
                        $1, $2, $3,
                        $4, $5, $6
                    ) ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind::<diesel::sql_types::Integer, _>(voteplan.id)
                .bind::<diesel::sql_types::VarChar, _>(&voteplan.chain_voteplan_id)
                .bind::<diesel::sql_types::Text, _>(&voteplan.chain_voteplan_payload)
                .bind::<diesel::sql_types::Integer, _>(&voteplan.fund_id)
                .bind::<diesel::sql_types::VarChar, _>(&voteplan.chain_vote_encryption_key)
                .bind::<diesel::sql_types::Integer, _>(&res.row_id)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
            }

            for (ix, fund_goal) in fund.goals.iter().enumerate() {
                diesel::sql_query(
                    r#"
                    INSERT INTO goal (
                        id,
                        name,
                        event_id,
                        idx
                    ) VALUES (
                        $1, $2, $3, $4
                    ) ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind::<diesel::sql_types::Integer, _>(&fund_goal.id)
                .bind::<diesel::sql_types::VarChar, _>(&fund_goal.goal_name)
                .bind::<diesel::sql_types::Integer, _>(&fund_goal.fund_id)
                .bind::<diesel::sql_types::Integer, _>(ix as i32)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
            }
        }

        Ok(())
    }

    pub fn insert_challenges(&self, challenges: &[Challenge]) -> Result<(), DbInserterError> {
        for challenge in challenges {
            diesel::sql_query(
                r#"
                INSERT INTO challenge (
                    row_id,
                    id,
                    category,
                    title,
                    description,
                    rewards_total,
                    proposers_rewards,
                    event,
                    extra
                ) VALUES (
                    $1, $2, $3,
                    $4, $5, $6,
                    $7, $8, $9
                )
            "#,
            )
            .bind::<diesel::sql_types::Integer, _>(challenge.internal_id)
            .bind::<diesel::sql_types::Integer, _>(challenge.id)
            .bind::<diesel::sql_types::Text, _>(challenge.challenge_type.to_string())
            .bind::<diesel::sql_types::Text, _>(&challenge.title)
            .bind::<diesel::sql_types::Text, _>(&challenge.description)
            .bind::<diesel::sql_types::BigInt, _>(challenge.rewards_total)
            .bind::<diesel::sql_types::BigInt, _>(challenge.proposers_rewards)
            .bind::<diesel::sql_types::Integer, _>(challenge.fund_id)
            .bind::<diesel::sql_types::Jsonb, _>(serde_json::json!({
                "url": {
                    "challenge": challenge.challenge_url,
                },
                "highlights": serde_json::to_string(&challenge.highlights).ok(),
            }))
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;
        }

        Ok(())
    }

    pub fn insert_advisor_reviews(&self, reviews: &[AdvisorReview]) -> Result<(), DbInserterError> {
        for review in reviews {
            diesel::sql_query(
                r#"
                INSERT INTO community_advisors_review (
                    proposal_id,
                    assessor,
                    impact_alignment_rating_given,
                    impact_alignment_note,
                    feasibility_rating_given,
                    feasibility_note,
                    auditability_rating_given,
                    auditability_note,
                    ranking
                ) VALUES (
                    $1, $2, $3,
                    $4, $5, $6,
                    $7, $8, $9
                )
            "#,
            )
            .bind::<diesel::sql_types::Integer, _>(review.proposal_id)
            .bind::<diesel::sql_types::VarChar, _>(&review.assessor)
            .bind::<diesel::sql_types::Integer, _>(review.impact_alignment_rating_given)
            .bind::<diesel::sql_types::VarChar, _>(&review.impact_alignment_note)
            .bind::<diesel::sql_types::Integer, _>(review.feasibility_rating_given)
            .bind::<diesel::sql_types::VarChar, _>(&review.feasibility_note)
            .bind::<diesel::sql_types::Integer, _>(review.auditability_rating_given)
            .bind::<diesel::sql_types::VarChar, _>(&review.auditability_note)
            .bind::<diesel::sql_types::Integer, _>(review.ranking as i32)
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }

    pub fn insert_groups(&self, groups: &[Group]) -> Result<(), DbInserterError> {
        for group in groups {
            diesel::sql_query(
                "INSERT INTO voting_group (event_id, token_id, group_id) VALUES ($1, $2, $3) ON CONFLICT (token_id, event_id) DO NOTHING",
            )
            .bind::<diesel::sql_types::Integer, _>(&group.fund_id)
            .bind::<diesel::sql_types::VarChar, _>(&group.token_identifier)
            .bind::<diesel::sql_types::VarChar, _>(&group.group_id)
            .execute(self.connection)?;
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum DbInserterError {
    #[error("internal diesel error")]
    DieselError(#[from] diesel::result::Error),
}
