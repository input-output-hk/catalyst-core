//! ## Database snapshot tests
//!
//! These tests use a standard database file, which can be found [here][db download]. To run these
//! tests create a file at `<project_root>/test_db.json` containing credentials for a postgres
//! database with this data. E.g.
//! ```json
//! {
//!   "host": "localhost",
//!   "name": "database_name",
//!   "user": "database_user",
//!   "password": "super secret password"
//! }
//! ```
//!
//! Each test runs a specific query and makes sure the output hasn't changed (i.e. a snapshot
//! test). It uses [`insta`][insta] as the snapshot testing library. Docs for how to use can be
//! found in the crate docs
//!
//! [db download]: https://updates-cardano-testnet.s3.amazonaws.com/cardano-db-sync/index.html#13/
//! [insta]: https://docs.rs/insta

use std::fs::File;

use crate::{model::SlotNo, Db};

fn test_db() -> Db {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_db.json");
    let reader = File::open(path).expect("couldn't read test_db.json");
    let config = serde_json::from_reader(reader).unwrap();

    Db::connect(config).expect("error connecting to database")
}

#[test]
fn default_query() {
    let db = test_db();
    let outputs = crate::voting_power(&db, None, None, None).unwrap();

    insta::assert_json_snapshot!(outputs);
}

#[test]
fn query_in_max_slot_range() {
    let lower_bound = Some(SlotNo(0));
    let upper_bound = Some(SlotNo(i64::MAX as u64));

    let db = test_db();
    let outputs = crate::voting_power(&db, lower_bound, upper_bound, None).unwrap();

    insta::assert_json_snapshot!(outputs);
}
