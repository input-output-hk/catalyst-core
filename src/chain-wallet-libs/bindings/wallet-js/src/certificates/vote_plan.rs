use std::convert::TryFrom;

use chain_impl_mockchain::certificate::VotePlanId as VotePlanIdLib;
use wasm_bindgen::prelude::*;

// TODO add VotePlan certificate

#[derive(Clone, Debug, PartialEq, Eq)]
#[wasm_bindgen]
pub struct VotePlanId(pub(crate) VotePlanIdLib);

#[wasm_bindgen]
impl VotePlanId {
    pub fn from_bytes(bytes: &[u8]) -> Result<VotePlanId, JsValue> {
        Ok(VotePlanId(
            VotePlanIdLib::try_from(bytes).map_err(|e| JsValue::from(e.to_string()))?,
        ))
    }

    pub fn to_bytes(&self) -> Result<Box<[u8]>, JsValue> {
        Ok(self.0.as_ref().into())
    }
}
