use wallet_wasm_js::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn basic_vote_id() {
    let bytes = core::array::from_fn::<u8, 32, _>(|i| i as u8);
    let vote_plan_id = VotePlanId::from_bytes(&bytes).unwrap();
    let decoded_bytes = vote_plan_id.to_bytes().unwrap();

    let bytes: Box<[u8]> = bytes.into();
    assert_eq!(bytes, decoded_bytes);
}
