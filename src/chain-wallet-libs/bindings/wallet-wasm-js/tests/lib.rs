//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

mod certificates;

wasm_bindgen_test_configure!(run_in_browser);

const PRIVATE_KEY: &str = "c86596c2d1208885db1fe3658406aa0f7cc7b8e13c362fe46a6db277fc5064583e487588c98a6c36e2e7445c0add36f83f171cb5ccfd815509d19cd38ecb0af3";
const ACCOUNT_ID: &str = "a6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663";
const SETTINGS: &str = r#"{"fees":{"constant":10,"coefficient":2,"certificate":100},"discrimination":"production","block0_initial_hash":{"hash":"baf6b54817cf2a3e865f432c3922d28ac5be641e66662c66d445f141e409183e"},"block0_date":1586637936,"slot_duration":20,"time_era":{"epoch_start":0,"slot_start":0,"slots_per_epoch":180},"transaction_max_expiry_epochs":1}"#;
