use wallet_js::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn basic_vote_id() {
    let bytes = [
        0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];
    let vote_plan_id = VotePlanId::from_bytes(&bytes).unwrap();
    let decoded_bytes = vote_plan_id.to_bytes().unwrap();

    let bytes: Box<[u8]> = bytes.into();
    assert_eq!(bytes, decoded_bytes);
}
