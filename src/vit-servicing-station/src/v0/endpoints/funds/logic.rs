use crate::db::{
    models::{funds::Fund, voteplans::Voteplan},
    schema::{funds::dsl as fund_dsl, voteplans::dsl as voteplans_dsl},
};
use crate::v0::context::SharedContext;
use crate::v0::errors::HandleError;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn get_fund_by_id(id: i32, context: SharedContext) -> Result<Fund, HandleError> {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        let query_results = (
            diesel::QueryDsl::filter(fund_dsl::funds, fund_dsl::id.eq(id))
                .first::<Fund>(&db_conn)
                .map_err(|_e| HandleError::NotFound("Error loading fund".to_string())),
            diesel::QueryDsl::filter(voteplans_dsl::voteplans, voteplans_dsl::fund_id.eq(id))
                .load::<Voteplan>(&db_conn)
                .map_err(|_e| HandleError::NotFound("Error loading voteplans".to_string())),
        );
        match query_results {
            (Ok(mut fund), Ok(mut voteplans)) => {
                fund.chain_vote_plans.append(&mut voteplans);
                Ok(fund)
            }
            // Any other combination is not valid
            _ => Err(HandleError::NotFound(format!(
                "Error loading fund with id {}",
                id
            ))),
        }
    })
    .await
    .map_err(|_| HandleError::NotFound(format!("Error loading fund with id {}", id)))?
}

pub async fn get_fund(context: SharedContext) -> Result<Fund, HandleError> {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .map_err(HandleError::DatabaseError)?;
    // let (mut fund, mut vote_plans): (Fund, Vec<Voteplan>) =
    tokio::task::spawn_blocking(move || {
        let fund = fund_dsl::funds
            .first::<Fund>(&db_conn)
            .map_err(|_e| HandleError::NotFound("fund".to_string()));
        match fund {
            Ok(mut fund) => diesel::QueryDsl::filter(
                voteplans_dsl::voteplans,
                voteplans_dsl::fund_id.eq(fund.id),
            )
            .load::<Voteplan>(&db_conn)
            .map_err(|_e| HandleError::NotFound("fund voteplans".to_string()))
            .map(|mut voteplans| {
                fund.chain_vote_plans.append(&mut voteplans);
                Ok(fund)
            }),
            Err(e) => Err(e),
        }
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))??
}
