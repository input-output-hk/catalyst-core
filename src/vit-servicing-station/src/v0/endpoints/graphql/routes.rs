use super::schema::Query;
use crate::v0::context::SharedContext;
use async_graphql::{http::GQLResponse, EmptyMutation, EmptySubscription, QueryBuilder, Schema};
use std::convert::Infallible;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    _context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let schema = Schema::new(Query {}, EmptyMutation, EmptySubscription);

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
