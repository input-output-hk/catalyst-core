use crate::db::{models::proposals::FullProposalInfo, queries::proposals as proposals_queries};
use crate::v0::endpoints::proposals::requests::ProposalsByVoteplanIdAndIndex;
use crate::v0::{context::SharedContext, errors::HandleError};

pub async fn get_all_proposals(
    context: SharedContext,
) -> Result<Vec<FullProposalInfo>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    proposals_queries::query_all_proposals(pool).await
}

pub async fn get_proposal(
    id: i32,
    context: SharedContext,
) -> Result<FullProposalInfo, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    proposals_queries::query_proposal_by_id(id, pool).await
}

pub async fn get_proposals_by_voteplan_id_and_index(
    query_data: ProposalsByVoteplanIdAndIndex,
    context: SharedContext,
) -> Result<Vec<FullProposalInfo>, HandleError> {
    let pool = context.read().await.db_connection_pool.clone();
    let tasks: Vec<_> = query_data
        .into_iter()
        .map(|proposal_query| {
            tokio::spawn(
                proposals_queries::query_proposals_by_voteplan_id_and_indexes(
                    proposal_query.vote_plan_id,
                    proposal_query.indexes,
                    pool.clone(),
                ),
            )
        })
        .collect();
    let mut results = Vec::new();

    for task in tasks {
        results.push(
            task.await.map_err(|e| {
                HandleError::InternalError(format!("Error executing task: {:?}", e))
            })??,
        );
    }

    Ok(results
        .into_iter()
        .flat_map(IntoIterator::into_iter)
        .collect())
}
