//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wallet_js::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const BLOCK0: &[u8] = include_bytes!("../../../test-vectors/block0");
const WALLET_VALUE: u64 = 1000000 + 10000 + 10000 + 1 + 100;

/// test to recover a yoroi style address in the test-vectors block0
///
#[wasm_bindgen_test]
fn yoroi1() {
    let mut wallet = Wallet::recover("neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone", &[]).expect("couldn't recover wallet fully");
    let settings = wallet.retrieve_funds(BLOCK0).unwrap();
    assert_eq!(wallet.total_value(), WALLET_VALUE);

    let conversion = wallet.convert(&settings);

    assert_eq!(conversion.num_ignored(), 1);
    assert_eq!(conversion.total_value_ignored(), 1);
    assert_eq!(conversion.transactions_len(), 1);

    let _transaction_bytes = conversion
        .transactions_get(0)
        .expect("to get the only transaction present in the conversion");
}

#[wasm_bindgen_test]
fn gen_key() {
    // just test that the random generator works
    let _key = Ed25519ExtendedPrivate::generate();
}

#[wasm_bindgen_test]
fn encrypt_decrypt() {
    let data = [1u8; 64 * 2];
    let password = [1u8, 2, 3, 4];
    let encrypted = symmetric_encrypt(&password, &data).unwrap();
    assert_eq!(
        &symmetric_decrypt(&password, &encrypted).unwrap()[..],
        &data[..]
    );
}

#[wasm_bindgen_test]
fn gen_key_from_seed() {
    let seed1 = [1u8; 32];
    let key1 = Ed25519ExtendedPrivate::from_seed(seed1.as_ref()).unwrap();
    let key2 = Ed25519ExtendedPrivate::from_seed(seed1.as_ref()).unwrap();

    assert_eq!(key1.bytes(), key2.bytes());

    let seed2 = [2u8; 32];
    let key3 = Ed25519ExtendedPrivate::from_seed(seed2.as_ref()).unwrap();

    assert_ne!(key3.bytes(), key1.bytes());
}

#[wasm_bindgen_test]
fn gen_key_from_invalid_seed_fails() {
    const INVALID_SEED_SIZE: usize = 32 + 1;
    let bad_seed = [2u8; INVALID_SEED_SIZE];
    assert!(Ed25519ExtendedPrivate::from_seed(bad_seed.as_ref()).is_err())
}

#[wasm_bindgen_test]
fn sign_verify_extended() {
    let key = Ed25519ExtendedPrivate::generate();
    let msg = [1, 2, 3, 4u8];
    let signature = key.sign(&msg);

    assert!(key.public().verify(&signature, &msg));
}

#[wasm_bindgen_test]
fn sign_verify() {
    let key = Ed25519Private::generate();
    let msg = [1, 2, 3, 4u8];
    let signature = key.sign(&msg);

    assert!(key.public().verify(&signature, &msg));
}

#[wasm_bindgen_test]
fn test_base32_to_base256() {
    let sk = base32_to_base256(&[
        24u8, 1, 3, 0, 26, 8, 23, 1, 19, 27, 30, 30, 31, 31, 29, 5, 31, 25, 20, 30, 10, 25, 7, 16,
        30, 15, 23, 26, 15, 4, 27, 11, 29, 28, 22, 21, 18, 28, 24, 8, 3, 9, 2, 4, 16, 15, 23, 27,
        1, 19, 6, 28, 29, 28, 30, 3, 24, 21, 23, 25, 29, 29, 30, 15, 12, 21, 23, 22, 9, 0, 10, 6,
        3, 10, 3, 30, 9, 0, 16, 4, 22, 24, 14, 6, 9, 8, 28, 19, 7, 26, 14, 4, 1, 29, 13, 29, 4, 19,
        2, 10, 24, 30, 0,
    ])
    .unwrap();

    assert_eq!(sk.len(), 64);
}
