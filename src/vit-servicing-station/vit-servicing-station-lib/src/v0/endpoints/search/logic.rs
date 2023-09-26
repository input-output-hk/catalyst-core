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
    use crate::testing::filters::ResponseBytesExt;
    use crate::v0::context::test::new_test_shared_context_from_url;
    use crate::v0::endpoints::search::requests::{Column, Constraint, OrderBy, Table};
    use pretty_assertions::assert_eq;
    use vit_servicing_station_tests::common::{
        data::ArbitrarySnapshotGenerator, startup::db::DbBuilder,
    };
    use warp::Filter;

    #[tokio::test]
    async fn basic_search() {
        // build context
        let mut gen = ArbitrarySnapshotGenerator::default();

        let mut snapshot = gen.snapshot();
        let c = snapshot.challenges_mut();
        c[0].title = "abc1".to_string();

        let db_url = DbBuilder::new()
            .with_snapshot(&snapshot)
            .build_async()
            .await
            .unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

        let challenge = snapshot.challenges().remove(0);

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
        assert_eq!(
            serde_json::to_value(&challenges[0]).unwrap(),
            serde_json::to_value(&challenge).unwrap()
        );

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

    /* TODO: Find out why this fails, if we don't obsolete vitSS soon enough. */
    #[tokio::test]
    async fn multiple_item_search() {
        // build context
        let mut gen = ArbitrarySnapshotGenerator::default();

        let mut snapshot = gen.snapshot();
        let c = snapshot.challenges_mut();
        c[0].title = "abc1".to_string();
        c[1].title = "abcd1".to_string();
        c[2].title = "abcde1".to_string();

        let db_url = DbBuilder::new()
            .with_snapshot(&snapshot)
            .build_async()
            .await
            .unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

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

        let expected_challenges = snapshot
            .challenges()
            .into_iter()
            .take(3)
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(&expected_challenges).unwrap(),
            serde_json::to_value(&challenges).unwrap()
        );

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

        let expected_challenges = snapshot
            .challenges()
            .into_iter()
            .take(3)
            .rev()
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(&expected_challenges).unwrap(),
            serde_json::to_value(&reversed).unwrap()
        );

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
        // build context
        let mut gen = ArbitrarySnapshotGenerator::default();

        let mut snapshot = gen.snapshot();
        let c = snapshot.challenges_mut();
        c[0].title = "abc1".to_string();
        c[1].title = "abcd1".to_string();
        c[2].title = "abcd1".to_string();
        c[3].title = "abcd1".to_string();
        c[4].title = "abcde1".to_string();

        let db_url = DbBuilder::new()
            .with_snapshot(&snapshot)
            .build_async()
            .await
            .unwrap();
        let shared_context = new_test_shared_context_from_url(&db_url);
        let filter_context = shared_context.clone();
        let with_context = warp::any().map(move || filter_context.clone());

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

        let expected_challenges = snapshot
            .challenges()
            .into_iter()
            .skip(1)
            .take(4)
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(expected_challenges).unwrap(),
            serde_json::to_value(challenges).unwrap()
        );
    }
}
