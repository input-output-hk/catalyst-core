use super::vote_plan::VotePlanId;
use chain_impl_mockchain::{
    certificate::VoteCast as VoteCastLib,
    vote::{Choice, Payload as PayloadLib},
};
use rand_chacha::rand_core::SeedableRng;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Payload(pub(crate) PayloadLib);

#[wasm_bindgen]
impl Payload {
    pub fn new_public(choice: u8) -> Payload {
        Self(PayloadLib::Public {
            choice: Choice::new(choice),
        })
    }

    pub fn new_private(
        vote_plan: VotePlanId,
        options: usize,
        choice: u8,
        public_key: &[u8],
    ) -> Result<Payload, JsValue> {
        let mut rng = rand_chacha::ChaChaRng::from_entropy();

        let public_key = chain_vote::ElectionPublicKey::from_bytes(public_key)
            .ok_or_else(|| JsValue::from_str("Cannot parse public key bytes"))?;

        let vote = chain_vote::Vote::new(options, choice as usize);
        let crs = chain_vote::Crs::from_hash(vote_plan.0.as_ref());
        let (encrypted_vote, proof) =
            chain_impl_mockchain::vote::encrypt_vote(&mut rng, &crs, &public_key, vote);

        Ok(Self(PayloadLib::Private {
            encrypted_vote,
            proof,
        }))
    }
}

#[wasm_bindgen]
pub struct VoteCast(pub(crate) VoteCastLib);

#[wasm_bindgen]
impl VoteCast {
    pub fn new(vote_plan: VotePlanId, proposal_index: u8, payload: Payload) -> Self {
        Self(VoteCastLib::new(vote_plan.0, proposal_index, payload.0))
    }
}
