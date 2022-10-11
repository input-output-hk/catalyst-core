//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use std::assert_eq;

use wallet_js::{SpendingCounter, *};
use wasm_bindgen_test::*;

mod certificates;

wasm_bindgen_test_configure!(run_in_browser);

const ACCOUNT_KEY: &str = include_str!("../../../test-vectors/free_keys/key1.prv");
const BLOCK0: &[u8] = include_bytes!("../../../test-vectors/block0");
const MAX_LANES: usize = 8;

#[wasm_bindgen_test]
fn vote_cast_test() {
    let mut wallet = Wallet::import_keys(
        hex::decode(String::from(ACCOUNT_KEY).trim())
            .unwrap()
            .as_ref(),
    )
    .unwrap();

    assert_eq!(wallet.total_value(), 0);

    wallet
        .set_state(
            1_000,
            (0..MAX_LANES)
                .map(|lane| SpendingCounter::new(lane, 1))
                .collect::<Vec<SpendingCounter>>()
                .into(),
        )
        .unwrap();

    assert_eq!(wallet.total_value(), 1_000);

    let settings = Settings::new(BLOCK0).unwrap();

    let vote_cast = VoteCast::build_public(
        VotePlanId::from_bytes(&[
            0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ])
        .unwrap(),
        8,
        0,
    );

    let fragment = wallet
        .sign_transaction(
            &settings,
            BlockDate::new(0, 1),
            0,
            Certificate::vote_cast(vote_cast),
        )
        .unwrap();

    wallet.confirm_transaction(&fragment.id());
}
