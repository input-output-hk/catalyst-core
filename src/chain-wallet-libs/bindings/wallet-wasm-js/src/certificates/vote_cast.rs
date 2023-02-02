use super::vote_plan::VotePlanId;
use chain_impl_mockchain::{
    certificate::VoteCast as VoteCastLib,
    vote::{Choice, Payload as PayloadLib},
};
use rand_chacha::rand_core::SeedableRng;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ElectionPublicKey(chain_vote::ElectionPublicKey);

#[wasm_bindgen]
impl ElectionPublicKey {
    pub fn from_hex(hex_data: String) -> Result<ElectionPublicKey, JsValue> {
        Ok(ElectionPublicKey(
            chain_vote::ElectionPublicKey::from_bytes(
                hex::decode(hex_data)
                    .map_err(|e| JsValue::from(e.to_string()))?
                    .as_slice(),
            )
            .ok_or_else(|| JsValue::from_str("Cannot parse public key bytes"))?,
        ))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<ElectionPublicKey, JsValue> {
        Ok(ElectionPublicKey(
            chain_vote::ElectionPublicKey::from_bytes(bytes)
                .ok_or_else(|| JsValue::from_str("Cannot parse public key bytes"))?,
        ))
    }
}

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
        vote_plan: &VotePlanId,
        options: usize,
        choice: u8,
        public_key: &ElectionPublicKey,
    ) -> Result<Payload, JsValue> {
        let mut rng = rand_chacha::ChaChaRng::from_entropy();

        let vote = chain_vote::Vote::new(options, choice as usize)
            .map_err(|e| JsValue::from_str(e.to_string().as_str()))?;

        let crs = chain_vote::Crs::from_hash(vote_plan.0.as_ref());
        let (encrypted_vote, proof) =
            chain_impl_mockchain::vote::encrypt_vote(&mut rng, &crs, &public_key.0, vote);

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
