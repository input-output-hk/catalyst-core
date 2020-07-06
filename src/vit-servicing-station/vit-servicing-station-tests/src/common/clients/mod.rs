pub mod graphql;
pub mod rest;

pub use graphql::GraphqlClient;
pub use rest::{RestClient, RestError};
