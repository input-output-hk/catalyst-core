use serde::Deserialize;
use warp::{Rejection, Reply};

use crate::{
    db::queries::search::{search_db, SearchColumn, SearchSort, SearchTable},
    v0::{context::SharedContext, result::HandlerResult},
};

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) struct SearchQuery {
    table: SearchTable,
    column: SearchColumn,
    sort: Option<SearchSort>,
    query: String,
}

pub(super) async fn search(
    SearchQuery {
        table,
        column,
        sort,
        query,
    }: SearchQuery,
    ctx: SharedContext,
) -> Result<impl Reply, Rejection> {
    let pool = ctx.read().await.db_connection_pool.clone();
    Ok(HandlerResult(
        search_db(table, column, sort, query, &pool).await,
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::models::challenges::test::populate_db_with_challenge_conn;
    use crate::db::models::challenges::Challenge;
    use crate::db::models::proposals::test::populate_db_with_proposal_conn;
    use crate::testing::filters::ResponseBytesExt;
    use crate::{
        db::models::{
            challenges::test::get_test_challenge_with_fund_id, proposals::test::get_test_proposal,
        },
        testing::filters::test_context,
    };
    use tracing_test::traced_test;
    use warp::Filter;

    #[tokio::test]
    #[traced_test]
    async fn test_search() {
        let proposal = get_test_proposal();
        let challenge = get_test_challenge_with_fund_id(proposal.proposal.fund_id);

        let (with_context, conn) = test_context().await;
        populate_db_with_proposal_conn(&proposal, &conn);
        populate_db_with_challenge_conn(&challenge, &conn);

        let filter = warp::path!("search")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(search);

        let body = serde_json::to_string(&SearchQuery {
            table: SearchTable::Challenge,
            column: SearchColumn::ChallengeDesc,
            sort: None,
            query: "desc".to_string(),
        })
        .unwrap();

        let challenges: Vec<Challenge> = warp::test::request()
            .method("POST")
            .path("/search")
            .body(body)
            .reply(&filter)
            .await
            .as_json();

        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], challenge);
    }
}
