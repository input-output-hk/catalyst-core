//! These tests run a local instance using the Rust implementation, and compare that with the
//! Haskell implementation running on https://snapshot-trigger-service-testnet.gov.iog.io

use tokio::runtime::Runtime;
use tracing_test::traced_test;

use self::local::{get_db_config, get_haskell, get_rust};

mod local;
mod network;

#[test]
#[traced_test]
fn haskell_rust_compare() {
    let cases = [
        // None,
        Some(1.into()),
        Some(10.into()),
        Some(100.into()),
        Some(1000.into()),
    ];

    let db = get_db_config().unwrap();

    for case in cases {
        info!("{case:?}");

        let rust = get_rust(&db, case).unwrap();
        let haskell = get_haskell(&db, case).unwrap();

        assert_eq!(rust, haskell);
    }
}
