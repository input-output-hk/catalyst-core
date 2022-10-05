//! Reference database tests
//!
//! These tests also require a local postgres instance, but do not require pre-set data. These
//! tests will create a reference database, and run queries against it.
//!
//! These tests are also snapshot tests, meaning the easiest way to run them is with `cargo insta`,
//! which is provided by `flake.nix`

use crate::voting_power;

use init::reference_db;

mod init;

#[test]
fn simple_query() {
    let db = reference_db();
    let simple_result = voting_power(db, None, None, None).unwrap();
    insta::assert_json_snapshot!(simple_result);
}
