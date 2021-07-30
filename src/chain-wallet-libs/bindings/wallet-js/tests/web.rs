//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wallet_js::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const BLOCK0: &[u8] = include_bytes!("../../../test-vectors/block0");

#[wasm_bindgen_test]
fn gen_key() {
    // just test that the random generator works
    let _key = Ed25519ExtendedPrivate::generate();
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
