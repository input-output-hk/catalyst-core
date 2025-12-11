use chain_impl_mockchain::certificate::Certificate as CertificateLib;
use wasm_bindgen::prelude::*;

pub mod vote_cast;
pub mod vote_plan;

#[wasm_bindgen]
#[allow(dead_code)]
pub struct Certificate(pub(crate) CertificateLib);

#[wasm_bindgen]
impl Certificate {
    pub fn vote_cast(vote_cast: vote_cast::VoteCast) -> Certificate {
        Certificate(CertificateLib::VoteCast(vote_cast.0))
    }
}
