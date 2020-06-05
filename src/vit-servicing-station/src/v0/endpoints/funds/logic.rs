use crate::db::{
    models::{funds::Fund, voteplans::Voteplan},
    schema::{funds::dsl as fund_dsl, voteplans::dsl as voteplans_dsl},
};
use crate::v0::context::SharedContext;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn get_fund_by_id(id: i32, context: SharedContext) -> Fund {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .expect("Error connecting to database");
    let (mut fund, mut vote_plans): (Fund, Vec<Voteplan>) =
        tokio::task::spawn_blocking(move || {
            (
                diesel::QueryDsl::filter(fund_dsl::funds, fund_dsl::id.eq(id))
                    .first::<Fund>(&db_conn)
                    .expect("Error loading fund"),
                diesel::QueryDsl::filter(voteplans_dsl::voteplans, voteplans_dsl::fund_id.eq(id))
                    .load::<Voteplan>(&db_conn)
                    .expect("Error loading fund"),
            )
        })
        .await
        .expect("Error loading fund");

    fund.chain_vote_plans.append(&mut vote_plans);
    fund
}

pub async fn get_fund(context: SharedContext) -> Fund {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .expect("Error connecting to database");
    let (mut fund, mut vote_plans): (Fund, Vec<Voteplan>) =
        tokio::task::spawn_blocking(move || {
            let fund = fund_dsl::funds
                .first::<Fund>(&db_conn)
                .expect("Error loading fund");
            let voteplans = diesel::QueryDsl::filter(
                voteplans_dsl::voteplans,
                voteplans_dsl::fund_id.eq(fund.id),
            )
            .load::<Voteplan>(&db_conn)
            .expect("Error loading fund");
            (fund, voteplans)
        })
        .await
        .expect("Error loading fund");

    fund.chain_vote_plans.append(&mut vote_plans);
    fund
}
