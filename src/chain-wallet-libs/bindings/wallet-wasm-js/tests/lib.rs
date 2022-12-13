//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use std::assert_eq;

use chain_impl_mockchain::accounting::account::SpendingCounterIncreasing;
use wallet_js::{SpendingCounter, *};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

mod certificates;

wasm_bindgen_test_configure!(run_in_browser);

const ACCOUNT_KEY: &str = include_str!("../../../test-vectors/free_keys/key1.prv");
const BLOCK0: &[u8] = include_bytes!("../../../test-vectors/block0");

pub fn generate_wallet() -> Result<Wallet, JsValue> {
    let mut wallet = Wallet::import_keys(
        hex::decode(String::from(ACCOUNT_KEY).trim())
            .unwrap()
            .as_ref(),
    )?;

    wallet.set_state(
        1_000,
        (0..SpendingCounterIncreasing::LANES)
            .map(|lane| SpendingCounter::new(lane, 1))
            .collect::<Vec<SpendingCounter>>()
            .into(),
    )?;

    assert_eq!(wallet.total_value(), 1_000);
    Ok(wallet)
}

pub fn generate_settings() -> Result<Settings, JsValue> {
    Settings::new(BLOCK0)
}
