use super::schema::QueryRoot;
use crate::v0::context::SharedContext;
use async_graphql::{http::GQLResponse, EmptyMutation, EmptySubscription, QueryBuilder, Schema};
use std::convert::Infallible;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // load a connection pool for the graphql schema
    let db_connection_pool = context.clone().read().await.db_connection_pool.clone();

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
