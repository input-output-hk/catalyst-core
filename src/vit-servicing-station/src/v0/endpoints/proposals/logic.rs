use crate::db::{
    models::proposals::Proposal, views_schema::full_proposals_info::dsl as full_proposal_dsl,
    views_schema::full_proposals_info::dsl::full_proposals_info,
};
use crate::v0::context::SharedContext;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn get_all_proposals(context: SharedContext) -> Vec<Proposal> {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .expect("Error connecting to database");
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .load::<Proposal>(&db_conn)
            .expect("Error loading proposals")
    })
    .await
    .expect("Error loading proposals")
}

pub async fn get_proposal(id: i32, context: SharedContext) -> Proposal {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .expect("Error connecting to database");
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .filter(full_proposal_dsl::id.eq(id))
            .first::<Proposal>(&db_conn)
            .expect("Error loading proposals")
    })
    .await
    .expect("Error loading proposals")
}
