use crate::db::models::proposals::{community_choice, simple, FullProposalInfo, Proposal};
use crate::db::schema::proposals;
use crate::db::{
    schema::{
        proposal_community_choice_challenge as community_choice_proposal_dsl,
        proposal_simple_challenge as simple_proposal_dsl,
    },
    views_schema::full_proposals_info::dsl as full_proposal_dsl,
    views_schema::full_proposals_info::dsl::full_proposals_info,
    DbConnection, DbConnectionPool,
};
use crate::v0::errors::HandleError;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, Insertable, QueryResult, RunQueryDsl};

pub async fn query_all_proposals(
    pool: &DbConnectionPool,
) -> Result<Vec<FullProposalInfo>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .load::<FullProposalInfo>(&db_conn)
            .map_err(|_e| HandleError::NotFound("proposals".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_proposal_by_id(
    id: i32,
    pool: &DbConnectionPool,
) -> Result<FullProposalInfo, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .filter(full_proposal_dsl::id.eq(id))
            .first::<FullProposalInfo>(&db_conn)
            .map_err(|_e| HandleError::NotFound(format!("proposal with id {}", id)))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_proposals_by_voteplan_id_and_indexes(
    voteplan_id: String,
    indexes: Vec<i64>,
    pool: DbConnectionPool,
) -> Result<Vec<FullProposalInfo>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .filter(full_proposal_dsl::chain_voteplan_id.eq(voteplan_id.clone()))
            .filter(full_proposal_dsl::chain_proposal_index.eq_any(&indexes))
            .load::<FullProposalInfo>(&db_conn)
            .map_err(|_e| {
                HandleError::NotFound(format!(
                    "proposal with voteplan id {} and indexes {:?}",
                    voteplan_id, indexes
                ))
            })
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn insert_proposal(proposal: Proposal, db_conn: &DbConnection) -> QueryResult<usize> {
    diesel::insert_into(proposals::table)
        .values(proposal.values())
        .execute(db_conn)
}

pub fn batch_insert_proposals(
    proposals_slice: &[Proposal],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(proposals::table)
        .values(
            proposals_slice
                .iter()
                .cloned()
                .map(|proposal| proposal.values())
                .collect::<Vec<_>>(),
        )
        .execute(db_conn)
}

pub fn batch_insert_community_choice_challenge_data(
    values: &[community_choice::ChallengeSqlValues],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(community_choice_proposal_dsl::table)
        .values(values)
        .execute(db_conn)
}

pub fn batch_insert_simple_challenge_data(
    values: &[simple::ChallengeSqlValues],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(simple_proposal_dsl::table)
        .values(values)
        .execute(db_conn)
}
