use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::RunQueryDsl;
use diesel::SqliteConnection;
use thiserror::Error;
use vit_servicing_station_lib::db::{
    models::{api_tokens::APITokenData, proposals::Proposal},
    schema::{api_tokens, proposals, voteplans},
};

pub struct DbInserter<'a> {
    connection: &'a SqliteConnection,
}

impl<'a> DbInserter<'a> {
    pub fn new(connection: &'a SqliteConnection) -> Self {
        Self { connection }
    }

    pub fn insert_token(&self, token_data: &APITokenData) -> Result<(), DbInserterError> {
        let values = (
            api_tokens::dsl::token.eq(token_data.token.as_ref()),
            api_tokens::dsl::creation_time.eq(token_data.creation_time),
            api_tokens::dsl::expire_time.eq(token_data.expire_time),
        );

        diesel::insert_into(api_tokens::table)
            .values(values)
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;
        Ok(())
    }

    pub fn insert_proposals(&self, proposals: &[Proposal]) -> Result<(), DbInserterError> {
        for proposal in proposals {
            let values = (
                proposals::proposal_id.eq(proposal.proposal_id.clone()),
                proposals::proposal_category.eq(proposal.proposal_category.category_name.clone()),
                proposals::proposal_title.eq(proposal.proposal_title.clone()),
                proposals::proposal_summary.eq(proposal.proposal_summary.clone()),
                proposals::proposal_problem.eq(proposal.proposal_problem.clone()),
                proposals::proposal_solution.eq(proposal.proposal_solution.clone()),
                proposals::proposal_public_key.eq(proposal.proposal_public_key.clone()),
                proposals::proposal_funds.eq(proposal.proposal_funds),
                proposals::proposal_url.eq(proposal.proposal_url.clone()),
                proposals::proposal_files_url.eq(proposal.proposal_files_url.clone()),
                proposals::proposer_name.eq(proposal.proposer.proposer_name.clone()),
                proposals::proposer_contact.eq(proposal.proposer.proposer_email.clone()),
                proposals::proposer_url.eq(proposal.proposer.proposer_url.clone()),
                proposals::chain_proposal_id.eq(proposal.chain_proposal_id.clone()),
                proposals::chain_proposal_index.eq(proposal.chain_proposal_index),
                proposals::chain_vote_options.eq(proposal.chain_vote_options.as_csv_string()),
                proposals::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
            );
            diesel::insert_into(proposals::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;

            let voteplan_values = (
                voteplans::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
                voteplans::chain_vote_start_time.eq(proposal.chain_vote_start_time),
                voteplans::chain_vote_end_time.eq(proposal.chain_vote_end_time),
                voteplans::chain_committee_end_time.eq(proposal.chain_committee_end_time),
                voteplans::chain_voteplan_payload.eq(proposal.chain_voteplan_payload.clone()),
                voteplans::fund_id.eq(proposal.fund_id),
            );

            diesel::insert_into(voteplans::table)
                .values(voteplan_values)
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
