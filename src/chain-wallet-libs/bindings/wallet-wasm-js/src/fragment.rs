use chain_core::packer::Codec;
use chain_core::property::{DeserializeFromSlice, Serialize};
use chain_impl_mockchain::fragment::Fragment as FragmentLib;
use wasm_bindgen::prelude::*;

/// Identifier of a block fragment
#[wasm_bindgen]
#[allow(dead_code)]
pub struct FragmentId(pub(crate) wallet_core::FragmentId);

/// this is used only for giving the Array a type in the typescript generated notation
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Array<FragmentId>")]
    pub type FragmentIds;
}

#[wasm_bindgen]
pub struct Fragment(pub(crate) FragmentLib);

#[wasm_bindgen]
impl Fragment {
    pub fn id(&self) -> FragmentId {
        FragmentId(self.0.hash())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Fragment, JsValue> {
        Ok(Fragment(
            FragmentLib::deserialize_from_slice(&mut Codec::new(bytes))
                .map_err(|e| JsValue::from(e.to_string()))?,
        ))
    }

    pub fn to_bytes(&self) -> Result<Box<[u8]>, JsValue> {
        Ok(self
            .0
            .serialize_as_vec()
            .map_err(|e| JsValue::from(e.to_string()))?
            .into_boxed_slice())
    }
}
