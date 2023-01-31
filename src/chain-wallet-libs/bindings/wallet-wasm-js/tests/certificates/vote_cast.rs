use crate::{ACCOUNT_ID, PRIVATE_KEY, SETTINGS};
use wallet_wasm_js::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn vote_cast_public_test() {
    let settings = Settings::from_json(SETTINGS.to_string()).unwrap();

    let payload = Payload::new_public(0);

    let vote_cast = VoteCast::new(
        VotePlanId::from_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        )
        .unwrap(),
        2,
        payload,
    );
    let mut builder = VoteCastTxBuilder::new(settings, vote_cast);
    builder = builder.prepare_tx(ACCOUNT_ID.to_string()).unwrap();
    let _fragment = builder.sign_tx(PRIVATE_KEY.to_string()).unwrap();
}

#[wasm_bindgen_test]
fn vote_cast_public_test2() {
    let settings = Settings::from_json(SETTINGS.to_string()).unwrap();

    let payload = Payload::new_public(0);

    let vote_cast = VoteCast::new(
        VotePlanId::from_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        )
        .unwrap(),
        2,
        payload,
    );
    let mut builder = VoteCastTxBuilder::new(settings, vote_cast);
    builder = builder.prepare_tx(ACCOUNT_ID.to_string()).unwrap();
    let signature = "2195c6eca3e6901696e3c376cb01d27bca47ad13fe63d153e1883fef08921948960cb843fd3e8383a0cc3d15a47451cc9e3e1695fe0ebf0165a58a9d930c9d00";
    let _fragment = builder.build_tx(signature.to_string()).unwrap();
}

#[wasm_bindgen_test]
fn vote_cast_private_test() {
    let settings = Settings::from_json(SETTINGS.to_string()).unwrap();

    let payload = Payload::new_private(
        &VotePlanId::from_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        )
        .unwrap(),
        8,
        0,
        &ElectionPublicKey::from_hex(
            "bed88887abe0a84f64691fe0bdfa3daf1a6cd697a13f07ae07588910ce39c927".to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let vote_cast = VoteCast::new(
        VotePlanId::from_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        )
        .unwrap(),
        4,
        payload,
    );
    let mut builder = VoteCastTxBuilder::new(settings, vote_cast);
    builder = builder.prepare_tx(ACCOUNT_ID.to_string()).unwrap();
    let _fragment = builder.sign_tx(PRIVATE_KEY.to_string()).unwrap();
}

#[wasm_bindgen_test]
fn vote_cast_private_test2() {
    let settings = Settings::from_json(SETTINGS.to_string()).unwrap();

    let payload = Payload::new_private(
        &VotePlanId::from_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        )
        .unwrap(),
        8,
        0,
        &ElectionPublicKey::from_hex(
            "bed88887abe0a84f64691fe0bdfa3daf1a6cd697a13f07ae07588910ce39c927".to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let vote_cast = VoteCast::new(
        VotePlanId::from_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        )
        .unwrap(),
        4,
        payload,
    );
    let mut builder = VoteCastTxBuilder::new(settings, vote_cast);
    builder = builder.prepare_tx(ACCOUNT_ID.to_string()).unwrap();
    let signature = "a33524b702ff2371ee214981433d543a39fc4c08958ef58d29d1890ad611b4deca9b11d0fad5bc076a1e68b87d97410907337dbc94286b0819464092674e0508";
    let _fragment = builder.build_tx(signature.to_string()).unwrap();
}
