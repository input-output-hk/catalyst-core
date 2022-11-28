use warp::{Rejection, Reply};

use crate::{
    db::queries::search::{search_count_db, search_db},
    v0::{context::SharedContext, result::HandlerResult},
};

use super::requests::{SearchCountQuery, SearchQuery};

pub(super) async fn search(
    query: SearchQuery,
    ctx: SharedContext,
) -> Result<impl Reply, Rejection> {
    let pool = ctx.read().await.db_connection_pool.clone();
    Ok(HandlerResult(search_db(query, &pool).await))
}

pub(super) async fn search_count(
    query: SearchCountQuery,
    ctx: SharedContext,
) -> Result<impl Reply, Rejection> {
    let pool = ctx.read().await.db_connection_pool.clone();
    Ok(HandlerResult(search_count_db(query, &pool).await))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::models::challenges::Challenge;
    use crate::db::models::proposals::test::add_test_proposal_and_challenge;
    use crate::testing::filters::test_context;
    use crate::testing::filters::ResponseBytesExt;
    use crate::v0::endpoints::search::requests::Column;
    use crate::v0::endpoints::search::requests::Constraint;
    use crate::v0::endpoints::search::requests::OrderBy;
    use crate::v0::endpoints::search::requests::Table;
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
            .and(with_context.clone())
            .and_then(search);

        let body = serde_json::to_string(&SearchQuery {
            query: SearchCountQuery {
                table: Table::Challenges,
                filter: vec![Constraint::Text {
                    search: "1".to_string(),
                    column: Column::Title,
                }],
                order_by: vec![],
            },
            limit: None,
            offset: None,
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

        let body = serde_json::to_string(&SearchCountQuery {
            table: Table::Challenges,
            filter: vec![Constraint::Text {
                search: "1".to_string(),
                column: Column::Title,
            }],
            order_by: vec![],
        })
        .unwrap();

        let filter = warp::path!("search_count")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(search_count);

        let count: i64 = warp::test::request()
            .method("POST")
            .path("/search_count")
            .body(body)
            .reply(&filter)
            .await
            .as_json();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn multiple_item_search() {
        let (with_context, conn) = test_context().await;

        let (_, challenge_1) = add_test_proposal_and_challenge(1, &conn);
        let (_, challenge_2) = add_test_proposal_and_challenge(10, &conn);
        let (_, challenge_3) = add_test_proposal_and_challenge(12, &conn);
        add_test_proposal_and_challenge(20, &conn);

        let filter_search = warp::path!("search")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context.clone())
            .and_then(search);

        let filter_search_count = warp::path!("search_count")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(search_count);

        let query = SearchQuery {
            query: SearchCountQuery {
                table: Table::Challenges,
                filter: vec![Constraint::Text {
                    column: Column::Title,
                    search: "1".to_string(),
                }],
                order_by: vec![OrderBy::Column {
                    column: Column::Title,
                    descending: false,
                }],
            },
            limit: None,
            offset: None,
        };

        let body = serde_json::to_string(&query).unwrap();

        let challenges: Vec<Challenge> = warp::test::request()
            .method("POST")
            .path("/search")
            .body(body)
            .reply(&filter_search)
            .await
            .as_json();

        // db sets these fields
        let challenge_1 = Challenge {
            internal_id: 1,
            ..challenge_1
        };

        let challenge_2 = Challenge {
            internal_id: 2,
            ..challenge_2
        };

        let challenge_3 = Challenge {
            internal_id: 3,
            ..challenge_3
        };

        let output = vec![challenge_1, challenge_2, challenge_3];
        assert_eq!(challenges, output);

        let body = serde_json::to_string(&SearchCountQuery {
            table: Table::Challenges,
            filter: vec![Constraint::Text {
                column: Column::Title,
                search: "1".to_string(),
            }],
            order_by: vec![OrderBy::Column {
                column: Column::Title,
                descending: false,
            }],
        })
        .unwrap();

        let count: i64 = warp::test::request()
            .method("POST")
            .path("/search_count")
            .body(body)
            .reply(&filter_search_count)
            .await
            .as_json();

        assert_eq!(count, 3);

        let body = serde_json::to_string(&SearchQuery {
            query: SearchCountQuery {
                order_by: vec![OrderBy::Column {
                    column: Column::Title,
                    descending: true,
                }],
                ..query.query
            },
            ..query
        })
        .unwrap();

        let reversed: Vec<Challenge> = warp::test::request()
            .method("POST")
            .path("/search")
            .body(body)
            .reply(&filter_search)
            .await
            .as_json();

        let reversed_output = {
            let mut temp = output.clone();
            temp.reverse();
            temp
        };
        assert_eq!(reversed, reversed_output);

        let body = serde_json::to_string(&SearchCountQuery {
            order_by: vec![OrderBy::Column {
                column: Column::Title,
                descending: true,
            }],
            table: Table::Challenges,
            filter: vec![Constraint::Text {
                column: Column::Title,
                search: "1".to_string(),
            }],
        })
        .unwrap();

        let count: i64 = warp::test::request()
            .method("POST")
            .path("/search_count")
            .body(body)
            .reply(&filter_search_count)
            .await
            .as_json();

        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn limits_and_offset_item_search() {
        let (with_context, conn) = test_context().await;

        add_test_proposal_and_challenge(10, &conn);
        let (_, challenge_2) = add_test_proposal_and_challenge(11, &conn);
        let (_, challenge_3) = add_test_proposal_and_challenge(12, &conn);
        let (_, challenge_4) = add_test_proposal_and_challenge(13, &conn);
        let (_, challenge_5) = add_test_proposal_and_challenge(14, &conn);

        let filter = warp::path!("search")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context)
            .and_then(search);

        let query = SearchQuery {
            query: SearchCountQuery {
                table: Table::Challenges,
                filter: vec![Constraint::Text {
                    column: Column::Title,
                    search: "1".to_string(),
                }],
                order_by: vec![OrderBy::Column {
                    column: Column::Title,
                    descending: false,
                }],
            },
            limit: Some(4),
            offset: Some(1),
        };

        let body = serde_json::to_string(&query).unwrap();

        let challenges: Vec<Challenge> = warp::test::request()
            .method("POST")
            .path("/search")
            .body(body)
            .reply(&filter)
            .await
            .as_json();

        let challenge_2 = Challenge {
            internal_id: 2,
            ..challenge_2
        };

        let challenge_3 = Challenge {
            internal_id: 3,
            ..challenge_3
        };

        let challenge_4 = Challenge {
            internal_id: 4,
            ..challenge_4
        };

        let challenge_5 = Challenge {
            internal_id: 5,
            ..challenge_5
        };

        let output = vec![challenge_2, challenge_3, challenge_4, challenge_5];
        assert_eq!(challenges, output);
    }
}
