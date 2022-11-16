use std::convert::Infallible;

use serde::Deserialize;
use warp::{
    hyper::{body::Bytes, Response},
    Filter,
};

use crate::{
    db::{migrations::initialize_db_with_migration, DbConnection},
    v0::context::{test::new_db_test_shared_context, SharedContext},
};

/// Initialize an in-memory database with migrations and return a tuple containing:
///  - a context backed by that database
///  - a connection to the database
pub async fn test_context() -> (
    impl Filter<Extract = (SharedContext,), Error = Infallible> + Clone,
    DbConnection,
) {
    let shared_context = new_db_test_shared_context();

    let conn = shared_context
        .read()
        .await
        .db_connection_pool
        .get()
        .unwrap();
    initialize_db_with_migration(&conn).unwrap();
    (warp::any().map(move || shared_context.clone()), conn)
}

/// Extension trait for `Response<Bytes>` to make extracting body easier
pub trait ResponseBytesExt {
    fn as_str(&self) -> &str;

    fn as_json<'a, T>(&'a self) -> T
    where
        T: Deserialize<'a>;
}

impl ResponseBytesExt for Response<Bytes> {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self.body().as_ref()).unwrap()
    }

    fn as_json<'a, T>(&'a self) -> T
    where
        T: Deserialize<'a>,
    {
        let s = self.as_str();
        serde_json::from_str(s).unwrap()
    }
}
