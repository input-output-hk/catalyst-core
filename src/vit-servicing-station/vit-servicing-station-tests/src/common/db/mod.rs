use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::RunQueryDsl;
use diesel::{Insertable, QueryDsl};
use thiserror::Error;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::goals::InsertGoal;
use vit_servicing_station_lib::db::models::groups::Group;
use vit_servicing_station_lib::db::schema::goals;
use vit_servicing_station_lib::db::schema::{groups, proposals_voteplans};
use vit_servicing_station_lib::db::DbConnection;
use vit_servicing_station_lib::db::{
    models::{
        api_tokens::ApiTokenData,
        challenges::Challenge,
        funds::Fund,
        proposals::{FullProposalInfo, ProposalChallengeInfo},
    },
    schema::{
        api_tokens, challenges, community_advisors_reviews, funds,
        proposal_community_choice_challenge, proposal_simple_challenge, proposals, voteplans,
    },
};
use vit_servicing_station_lib::q;

macro_rules! insert_or_ignore_into_q {
    ($conn:ident,$table:expr,$values:expr) => {
        match $conn {
            DbConnection::Sqlite($conn) => diesel::insert_or_ignore_into($table)
                .values($values)
                .execute($conn),
            DbConnection::Postgres($conn) => diesel::insert_into($table)
                .values($values)
                .on_conflict_do_nothing()
                .execute($conn),
        }
    };
}

pub struct DbInserter<'a> {
    connection: &'a DbConnection,
}

impl<'a> DbInserter<'a> {
    pub fn new(connection: &'a DbConnection) -> Self {
        Self { connection }
    }

    pub fn insert_token(&self, token_data: &ApiTokenData) -> Result<(), DbInserterError> {
        let values = (
            api_tokens::dsl::token.eq(token_data.token.as_ref()),
            api_tokens::dsl::creation_time.eq(token_data.creation_time),
            api_tokens::dsl::expire_time.eq(token_data.expire_time),
        );

        let conn = self.connection;
        q!(
            conn,
            diesel::insert_into(api_tokens::table)
                .values(values)
                .execute(conn)
        )
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
        let conn = self.connection;

        for proposal in proposals {
            let proposal_id = proposal.proposal.proposal_id.clone();
            let values = (
                proposals::id.eq(proposal.proposal.internal_id),
                proposals::proposal_id.eq(proposal.proposal.proposal_id.clone()),
                proposals::proposal_category.eq(proposal
                    .proposal
                    .proposal_category
                    .category_name
                    .clone()),
                proposals::proposal_title.eq(proposal.proposal.proposal_title.clone()),
                proposals::proposal_summary.eq(proposal.proposal.proposal_summary.clone()),
                proposals::proposal_public_key.eq(proposal.proposal.proposal_public_key.clone()),
                proposals::proposal_funds.eq(proposal.proposal.proposal_funds),
                proposals::proposal_url.eq(proposal.proposal.proposal_url.clone()),
                proposals::proposal_files_url.eq(proposal.proposal.proposal_files_url.clone()),
                proposals::proposer_name.eq(proposal.proposal.proposer.proposer_name.clone()),
                proposals::proposer_contact.eq(proposal.proposal.proposer.proposer_email.clone()),
                proposals::proposer_url.eq(proposal.proposal.proposer.proposer_url.clone()),
                proposals::proposal_impact_score.eq(proposal.proposal.proposal_impact_score),
                proposals::proposer_relevant_experience.eq(proposal
                    .proposal
                    .proposer
                    .proposer_relevant_experience
                    .clone()),
                proposals::chain_proposal_id.eq(proposal.proposal.chain_proposal_id.clone()),
                proposals::chain_vote_options
                    .eq(proposal.proposal.chain_vote_options.as_csv_string()),
                proposals::challenge_id.eq(proposal.proposal.challenge_id),
                proposals::proposal_extra_fields
                    .eq(serde_json::to_string(&proposal.proposal.proposal_extra_fields).unwrap()),
            );

            insert_or_ignore_into_q!(conn, proposals::table, values)
                .map_err(DbInserterError::DieselError)?;

            let values = (
                proposals_voteplans::proposal_id.eq(proposal_id),
                proposals_voteplans::chain_proposal_index
                    .eq(proposal.voteplan.chain_proposal_index),
                proposals_voteplans::chain_voteplan_id
                    .eq(proposal.voteplan.chain_voteplan_id.clone()),
            );
            q!(
                conn,
                diesel::insert_into(proposals_voteplans::table)
                    .values(values)
                    .execute(conn)
            )
            .map_err(DbInserterError::DieselError)?;

            let token_id = q!(
                conn,
                groups::table
                    .filter(groups::fund_id.eq(proposal.proposal.fund_id))
                    .filter(groups::group_id.eq(&proposal.group_id))
                    .select(groups::token_identifier)
                    .first::<String>(conn)
            )
            .map_err(DbInserterError::DieselError)?;

            let voteplan_values = (
                voteplans::chain_voteplan_id.eq(proposal.voteplan.chain_voteplan_id.clone()),
                voteplans::chain_vote_start_time.eq(proposal.proposal.chain_vote_start_time),
                voteplans::chain_vote_end_time.eq(proposal.proposal.chain_vote_end_time),
                voteplans::chain_committee_end_time.eq(proposal.proposal.chain_committee_end_time),
                voteplans::chain_voteplan_payload
                    .eq(proposal.proposal.chain_voteplan_payload.clone()),
                voteplans::chain_vote_encryption_key
                    .eq(proposal.proposal.chain_vote_encryption_key.clone()),
                voteplans::fund_id.eq(proposal.proposal.fund_id),
                voteplans::token_identifier.eq(token_id),
            );

            insert_or_ignore_into_q!(conn, voteplans::table, voteplan_values)
                .map_err(DbInserterError::DieselError)?;

            match &proposal.challenge_info {
                ProposalChallengeInfo::Simple(data) => {
                    let simple_values = (
                        proposal_simple_challenge::proposal_id
                            .eq(proposal.proposal.proposal_id.clone()),
                        proposal_simple_challenge::proposal_solution
                            .eq(data.proposal_solution.clone()),
                    );
                    insert_or_ignore_into_q!(conn, proposal_simple_challenge::table, simple_values)
                        .map_err(DbInserterError::DieselError)?;
                }
                ProposalChallengeInfo::CommunityChoice(data) => {
                    let community_values = (
                        proposal_community_choice_challenge::proposal_id
                            .eq(proposal.proposal.proposal_id.clone()),
                        proposal_community_choice_challenge::proposal_brief
                            .eq(data.proposal_brief.clone()),
                        proposal_community_choice_challenge::proposal_importance
                            .eq(data.proposal_importance.clone()),
                        proposal_community_choice_challenge::proposal_goal
                            .eq(data.proposal_goal.clone()),
                        proposal_community_choice_challenge::proposal_metrics
                            .eq(data.proposal_metrics.clone()),
                    );

                    insert_or_ignore_into_q!(
                        conn,
                        proposal_community_choice_challenge::table,
                        community_values
                    )
                    .map_err(DbInserterError::DieselError)?;
                }
            };
        }
        Ok(())
    }

    pub fn insert_funds(&self, funds: &[Fund]) -> Result<(), DbInserterError> {
        let conn = self.connection;

        for fund in funds {
            let values = fund.clone().values();

            q!(
                conn,
                diesel::insert_into(funds::table)
                    .values(values)
                    .execute(conn)
            )
            .map_err(DbInserterError::DieselError)?;

            for voteplan in &fund.chain_vote_plans {
                let values = (
                    voteplans::id.eq(voteplan.id),
                    voteplans::chain_voteplan_id.eq(voteplan.chain_voteplan_id.clone()),
                    voteplans::chain_vote_start_time.eq(voteplan.chain_vote_start_time),
                    voteplans::chain_vote_end_time.eq(voteplan.chain_vote_end_time),
                    voteplans::chain_committee_end_time.eq(voteplan.chain_committee_end_time),
                    voteplans::chain_voteplan_payload.eq(voteplan.chain_voteplan_payload.clone()),
                    voteplans::chain_vote_encryption_key
                        .eq(voteplan.chain_vote_encryption_key.clone()),
                    voteplans::fund_id.eq(voteplan.fund_id),
                    voteplans::token_identifier.eq(voteplan.token_identifier.clone()),
                );
                insert_or_ignore_into_q!(conn, voteplans::table, values)
                    .map_err(DbInserterError::DieselError)?;
            }

            for goal in &fund.goals {
                insert_or_ignore_into_q!(conn, goals::table, InsertGoal::from(goal))
                    .map_err(DbInserterError::DieselError)?;
            }
        }
        Ok(())
    }

    pub fn insert_challenges(&self, challenges: &[Challenge]) -> Result<(), DbInserterError> {
        let conn = self.connection;

        for challenge in challenges {
            insert_or_ignore_into_q!(conn, challenges::table, challenge.clone().values())
                .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }

    pub fn insert_advisor_reviews(&self, reviews: &[AdvisorReview]) -> Result<(), DbInserterError> {
        let conn = self.connection;

        for review in reviews {
            insert_or_ignore_into_q!(
                conn,
                community_advisors_reviews::table,
                review.clone().values()
            )
            .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }

    pub fn insert_groups(&self, groups: &[Group]) -> Result<(), DbInserterError> {
        let conn = self.connection;

        for group in groups {
            insert_or_ignore_into_q!(conn, groups::table, group.clone().values())
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
