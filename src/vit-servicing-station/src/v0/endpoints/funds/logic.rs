use crate::db::models::vote_plan::ChainVoteplan;
use crate::db::{models::funds::Fund, schema::funds::dsl::*};
use crate::v0::context::{ChainData, SharedContext};
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn get_fund(name: String, context: SharedContext) -> Fund {
    // let db_conn = context
    //     .read()
    //     .await
    //     .db_connection_pool
    //     .get()
    //     .expect("Error connecting to database");
    // let fund : Fund = tokio::task::spawn_blocking(move || {
    //     funds
    //         .filter(fund_name.eq(name))
    //         .load::<Fund>(&db_conn)
    //         .expect("Error loading fund")
    // })
    // .await
    // .expect("Error loading fund");
    // let vote_plans : Vec<ChainVoteplan> = = tokio::task::spawn_blocking(move || {
    //     vote_plans
    //         .filter(fund_name.eq(name))
    //         .load::<Fund>(&db_conn)
    //         .expect("Error loading fund")
    // });
    Fund {
        fund_name: "".to_string(),
        fund_goal: "".to_string(),
        voting_power_info: "".to_string(),
        rewards_info: "".to_string(),
        fund_start_time: "".to_string(),
        fund_end_time: "".to_string(),
        next_fund_start_time: "".to_string(),
        chain_vote_plans: vec![],
    }
}
