//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wallet_js::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const WALLET_VALUE: u64 = 1000000 + 10000 + 10000 + 1 + 100;

/// test to recover a yoroi style address in the test-vectors block0
///
#[wasm_bindgen_test]
fn yoroi1() {
    let mut wallet = Wallet::recover("neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone", &[]).expect("couldn't recover wallet fully");
    let _settings = wallet.retrieve_funds(BLOCK0).unwrap();
    assert_eq!(wallet.total_value(), WALLET_VALUE);
}
