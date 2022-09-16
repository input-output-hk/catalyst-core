use crate::db::models::vote::Vote;
use crate::db::queries::votes as votes_queries;
use crate::v0::context::SharedContext;
use crate::v0::errors::HandleError;

pub async fn get_vote_by_caster_and_voteplan_id(
    caster: String,
    voteplan_id: String,
    context: SharedContext,
) -> Result<Vec<Vote>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    let votes =
        votes_queries::query_votes_by_caster_and_voteplan_id(caster, voteplan_id, pool).await?;
    Ok(votes)
}
