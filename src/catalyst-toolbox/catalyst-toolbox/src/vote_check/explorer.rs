use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "resources/explorer/transaction_by_id.graphql",
    schema_path = "resources/explorer/schema.graphql",
    response_derives = "Debug"
)]
pub struct TransactionById;
