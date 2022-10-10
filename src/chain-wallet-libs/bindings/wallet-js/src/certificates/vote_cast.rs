use chain_impl_mockchain::{
    certificate::{VoteCast as VoteCastLib, VotePlanId},
    vote::{Choice, Payload},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VoteCast(pub(crate) VoteCastLib);

impl VoteCast {
    pub fn build_public(vote_plan: VotePlanId, proposal_index: u8, choice: u8) -> Self {
        Self(VoteCastLib::new(
            vote_plan,
            proposal_index,
            Payload::Public {
                choice: Choice::new(choice),
            },
        ))
    }
}
