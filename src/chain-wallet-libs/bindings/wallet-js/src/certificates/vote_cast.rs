use chain_impl_mockchain::{
    certificate::{VoteCast as VoteCastLib, VotePlanId as VotePlanIdLib},
    vote::{Choice, Payload},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VotePlanId(pub(crate) VotePlanIdLib);

#[wasm_bindgen]
pub struct VoteCast(pub(crate) VoteCastLib);

#[wasm_bindgen]
impl VoteCast {
    pub fn build_public(vote_plan: VotePlanId, proposal_index: u8, choice: u8) -> Self {
        Self(VoteCastLib::new(
            vote_plan.0,
            proposal_index,
            Payload::Public {
                choice: Choice::new(choice),
            },
        ))
    }
}
