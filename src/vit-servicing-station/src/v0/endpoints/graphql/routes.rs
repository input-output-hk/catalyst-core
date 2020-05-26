use super::schema::QueryRoot;
use crate::db;
use crate::v0::context::SharedContext;
use async_graphql::{http::GQLResponse, EmptyMutation, EmptySubscription, QueryBuilder, Schema};
use std::convert::Infallible;
use std::sync::Arc;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    _context: SharedContext,
    db_url: &str,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // load a connection pool for the graphql schema
    let db_connection_pool =
        Arc::new(db::load_db_connection_pool(db_url).expect("Error connecting to database"));

    let schema = Schema::new(
        QueryRoot { db_connection_pool },
        EmptyMutation,
        EmptySubscription,
    );

    let graph_ql = async_graphql_warp::graphql(schema).and_then(
        |(schema, builder): (_, QueryBuilder)| async move {
            // Execute query
            let resp = builder.execute(&schema).await;
            // Return result
            Ok::<_, Infallible>(warp::reply::json(&GQLResponse(resp)).into_response())
        },
    );

    root.and(graph_ql).boxed()
}
