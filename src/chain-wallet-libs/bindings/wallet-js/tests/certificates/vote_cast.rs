use crate::{generate_settings, generate_wallet};
use wallet_js::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn vote_cast_public_test() {
    let mut wallet = generate_wallet().unwrap();

    let settings = generate_settings().unwrap();

    let vote_plan = VotePlanId::from_bytes(&[
        0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ])
    .unwrap();

    let vote_cast = VoteCast::new(vote_plan, 8, Payload::new_public(0));

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

#[wasm_bindgen_test]
fn vote_cast_private_test() {
    let mut wallet = generate_wallet().unwrap();

    let settings = generate_settings().unwrap();

    let vote_plan = VotePlanId::from_bytes(&[
        0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ])
    .unwrap();

    let vote_cast = VoteCast::new(
        vote_plan.clone(),
        8,
        Payload::new_private(
            vote_plan,
            4,
            0,
            &[
                4, 132, 125, 160, 220, 20, 185, 242, 239, 183, 52, 219, 9, 201, 17, 223, 218, 112,
                47, 41, 121, 93, 209, 8, 163, 232, 118, 17, 23, 6, 204, 235, 14, 205, 219, 21, 88,
                144, 31, 191, 20, 145, 69, 63, 111, 75, 68, 161, 2, 20, 1, 33, 237, 89, 204, 70,
                204, 219, 212, 22, 96, 102, 9, 103, 184,
            ],
        )
        .unwrap(),
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
