use std::collections::BTreeMap;

use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::{QueryDsl, RunQueryDsl};
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

// This is to make the schema definition (in Rust) be compiled
// using this crate's diesel version because election-db uses
// diesel 2.
mod election_db {
    pub mod schema {
        include!("../../../../../election-db/src/schema.rs");
    }
}

pub struct DbInserter<'a> {
    connection: &'a DbConnection,
}

impl<'a> DbInserter<'a> {
    pub fn new(connection: &'a DbConnection) -> Self {
        Self { connection }
    }

    pub fn insert_token(&self, token_data: &ApiTokenData) -> Result<(), DbInserterError> {
        use election_db::schema::config;

        let values = (
            config::id.eq("api_token"),
            config::id2.eq(base64::encode(token_data.token.as_ref())),
            config::id3.eq(""),
            config::value.eq(serde_json::json!({
                "created": token_data.creation_time,
                "expires": token_data.expire_time,
            })),
        );

        diesel::insert_into(config::table)
            .values(values)
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;

        Ok(())
    }

    pub fn insert_tokens(&self, tokens_data: &[ApiTokenData]) -> Result<(), DbInserterError> {
        for token_data in tokens_data {
            self.insert_token(token_data)?;
        }
        Ok(())
    }

    pub fn insert_proposals(&self, proposals: &[FullProposalInfo]) -> Result<(), DbInserterError> {
        use election_db::schema::{proposal, proposal_voteplan, voteplan};

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

            let values = (
                proposal::row_id.eq(proposal.proposal.internal_id),
                proposal::id.eq(proposal_id),
                proposal::title.eq(proposal.proposal.proposal_title.clone()),
                proposal::summary.eq(proposal.proposal.proposal_summary.clone()),
                proposal::category.eq(proposal.proposal.proposal_category.category_name.clone()),
                proposal::public_key.eq(proposal.proposal.proposal_public_key.clone()),
                proposal::funds.eq(proposal.proposal.proposal_funds),
                proposal::url.eq(proposal.proposal.proposal_url.clone()),
                proposal::files_url.eq(proposal.proposal.proposal_files_url.clone()),
                proposal::impact_score.eq(proposal.proposal.proposal_impact_score),
                proposal::proposer_name.eq(proposal.proposal.proposer.proposer_name.clone()),
                proposal::proposer_contact.eq(proposal.proposal.proposer.proposer_email.clone()),
                proposal::proposer_url.eq(proposal.proposal.proposer.proposer_url.clone()),
                proposal::proposer_relevant_experience.eq(proposal
                    .proposal
                    .proposer
                    .proposer_relevant_experience
                    .clone()),
                proposal::bb_proposal_id.eq(proposal.proposal.chain_proposal_id.clone()),
                proposal::bb_vote_options.eq(proposal.proposal.chain_vote_options.as_csv_string()),
                proposal::challenge.eq(proposal.proposal.challenge_id),
                proposal::extra.eq(serde_json::to_value(extra).unwrap()),
            );

            diesel::insert_into(proposal::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;

            let voteplan_row_id = voteplan::table
                .filter(voteplan::id.eq(&proposal.voteplan.chain_voteplan_id))
                .select(voteplan::row_id)
                .get_result::<i32>(self.connection)
                .unwrap();

            let values = (
                proposal_voteplan::proposal_id.eq(proposal.proposal.internal_id),
                proposal_voteplan::voteplan_id.eq(voteplan_row_id),
                proposal_voteplan::bb_proposal_index.eq(proposal.voteplan.chain_proposal_index),
            );

            diesel::insert_into(proposal_voteplan::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
        }

        Ok(())
    }

    pub fn insert_funds(&self, funds: &[Fund]) -> Result<(), DbInserterError> {
        use election_db::schema::{election, goal, voteplan, voting_group};

        for fund in funds {
            let id_item = if fund.id == 0 {
                None
            } else {
                Some(election::row_id.eq(fund.id))
            };

            let values = (
                id_item,
                election::name.eq(&fund.fund_name),
                election::description.eq(&fund.fund_goal),
                election::registration_snapshot_time.eq(
                    chrono::NaiveDateTime::from_timestamp_millis(
                        fund.registration_snapshot_time * 1000,
                    ),
                ),
                election::voting_power_threshold.eq(fund.voting_power_threshold),
                election::start_time.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.fund_start_time * 1000,
                )),
                election::end_time.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.fund_end_time * 1000,
                )),
                election::insight_sharing_start.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.insight_sharing_start * 1000,
                )),
                election::proposal_submission_start.eq(
                    chrono::NaiveDateTime::from_timestamp_millis(
                        fund.stage_dates.proposal_submission_start * 1000,
                    ),
                ),
                election::refine_proposals_start.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.refine_proposals_start * 1000,
                )),
                election::finalize_proposals_start.eq(
                    chrono::NaiveDateTime::from_timestamp_millis(
                        fund.stage_dates.finalize_proposals_start * 1000,
                    ),
                ),
                election::proposal_assessment_start.eq(
                    chrono::NaiveDateTime::from_timestamp_millis(
                        fund.stage_dates.proposal_assessment_start * 1000,
                    ),
                ),
                election::assessment_qa_start.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.assessment_qa_start * 1000,
                )),
                election::snapshot_start.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.snapshot_start * 1000,
                )),
                election::voting_start.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.voting_start * 1000,
                )),
                election::voting_end.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.voting_end * 1000,
                )),
                election::tallying_end.eq(chrono::NaiveDateTime::from_timestamp_millis(
                    fund.stage_dates.tallying_end * 1000,
                )),
                election::extra.eq(serde_json::json!({
                    "url": {
                        "results": fund.results_url,
                        "survey": fund.survey_url,
                    }
                })),
            );

            diesel::insert_into(election::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;

            self.insert_groups(&fund.groups.iter().cloned().collect::<Vec<_>>())?;

            for voteplan in &fund.chain_vote_plans {
                let group_row_id = voting_group::table
                    .filter(voting_group::token_id.eq(&voteplan.token_identifier))
                    .select(voting_group::row_id)
                    .get_result::<i32>(self.connection)
                    .unwrap();

                let values = (
                    voteplan::row_id.eq(voteplan.id),
                    voteplan::id.eq(&voteplan.chain_voteplan_id),
                    voteplan::category.eq(&voteplan.chain_voteplan_payload),
                    voteplan::election_id.eq(voteplan.fund_id),
                    voteplan::encryption_key.eq(&voteplan.chain_vote_encryption_key),
                    voteplan::group_id.eq(group_row_id),
                );

                diesel::insert_into(voteplan::table)
                    .values(values)
                    .on_conflict_do_nothing()
                    .execute(self.connection)
                    .map_err(DbInserterError::DieselError)?;
            }

            for (ix, fund_goal) in fund.goals.iter().enumerate() {
                diesel::insert_into(goal::table)
                    .values((
                        goal::id.eq(fund_goal.id),
                        goal::name.eq(&fund_goal.goal_name),
                        goal::election_id.eq(fund_goal.fund_id),
                        goal::idx.eq(ix as i32),
                    ))
                    .on_conflict_do_nothing()
                    .execute(self.connection)
                    .map_err(DbInserterError::DieselError)?;
            }
        }
        Ok(())
    }

    pub fn insert_challenges(&self, challenges: &[Challenge]) -> Result<(), DbInserterError> {
        use election_db::schema::challenge;

        for challenge in challenges {
            let values = (
                challenge::row_id.eq(challenge.internal_id),
                challenge::id.eq(challenge.id),
                challenge::category.eq(challenge.challenge_type.to_string()),
                challenge::title.eq(&challenge.title),
                challenge::description.eq(&challenge.description),
                challenge::rewards_total.eq(challenge.rewards_total),
                challenge::proposers_rewards.eq(challenge.proposers_rewards),
                challenge::election.eq(challenge.fund_id),
                challenge::extra.eq(serde_json::json!({
                    "url": {
                        "challenge": challenge.challenge_url,
                    },
                    "highlights": serde_json::to_string(&challenge.highlights).ok(),
                })),
            );

            diesel::insert_into(challenge::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }

    pub fn insert_advisor_reviews(&self, reviews: &[AdvisorReview]) -> Result<(), DbInserterError> {
        use election_db::schema::community_advisors_review;

        for review in reviews {
            let values = (
                community_advisors_review::proposal_id.eq(review.proposal_id),
                community_advisors_review::assessor.eq(&review.assessor),
                community_advisors_review::impact_alignment_rating_given
                    .eq(review.impact_alignment_rating_given),
                community_advisors_review::impact_alignment_note.eq(&review.impact_alignment_note),
                community_advisors_review::feasibility_rating_given
                    .eq(review.feasibility_rating_given),
                community_advisors_review::feasibility_note.eq(&review.feasibility_note),
                community_advisors_review::auditability_rating_given
                    .eq(review.auditability_rating_given),
                community_advisors_review::auditability_note.eq(&review.auditability_note),
                community_advisors_review::ranking.eq(review.ranking as i32),
            );

            diesel::insert_into(community_advisors_review::table)
                .values(values)
                .on_conflict_do_nothing()
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }

    pub fn insert_groups(&self, groups: &[Group]) -> Result<(), DbInserterError> {
        use election_db::schema::voting_group;

        for group in groups {
            let values = (
                voting_group::election_id.eq(group.fund_id),
                voting_group::token_id.eq(&group.token_identifier),
                voting_group::group_id.eq(&group.group_id),
            );

            diesel::insert_into(voting_group::table)
                .values(values)
                .on_conflict_do_nothing()
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum DbInserterError {
    #[error("internal diesel error")]
    DieselError(#[from] diesel::result::Error),
}
