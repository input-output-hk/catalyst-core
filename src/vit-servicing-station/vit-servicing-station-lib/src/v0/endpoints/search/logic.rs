use warp::{Rejection, Reply};

use crate::{
    db::queries::search::search_db,
    v0::{context::SharedContext, result::HandlerResult},
};

use super::requests::SearchRequest;

pub(super) async fn search(
    req: SearchRequest,
    ctx: SharedContext,
) -> Result<impl Reply, Rejection> {
    let pool = ctx.read().await.db_connection_pool.clone();
    Ok(HandlerResult(search_db(req, &pool).await))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::models::challenges::Challenge;
    use crate::db::models::proposals::test::add_test_proposal_and_challenge;
    use crate::testing::filters::test_context;
    use crate::testing::filters::ResponseBytesExt;
    use crate::v0::endpoints::search::requests::{
        SearchColumn, SearchRequest, SearchSort, SearchTable,
    };
    use warp::Filter;

    #[tokio::test]
    async fn basic_search() {
        let (with_context, conn) = test_context().await;

        let (_, challenge) = add_test_proposal_and_challenge(1, &conn);
        add_test_proposal_and_challenge(2, &conn);
        add_test_proposal_and_challenge(3, &conn);
        add_test_proposal_and_challenge(4, &conn);

        let filter = warp::path!("search")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(search);

        let body = serde_json::to_string(&SearchRequest {
            table: SearchTable::Challenge,
            column: SearchColumn::ChallengeTitle,
            sort: SearchSort::Index,
            query: "1".to_string(),
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

    #[tokio::test]
    async fn multiple_item_search() {
        let (with_context, conn) = test_context().await;

        let (_, challenge_1) = add_test_proposal_and_challenge(1, &conn);
        let (_, challenge_2) = add_test_proposal_and_challenge(10, &conn);
        let (_, challenge_3) = add_test_proposal_and_challenge(12, &conn);
        add_test_proposal_and_challenge(20, &conn);

        let filter = warp::path!("search")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(search);

        let body = serde_json::to_string(&SearchRequest {
            table: SearchTable::Challenge,
            column: SearchColumn::ChallengeDesc,
            sort: SearchSort::ChallengeTitle,
            query: "1".to_string(),
        })
        .unwrap();

        let challenges: Vec<Challenge> = warp::test::request()
            .method("POST")
            .path("/search")
            .body(body)
            .reply(&filter)
            .await
            .as_json();

        assert_eq!(challenges, vec![challenge_1, challenge_2, challenge_3]);
    }
}
