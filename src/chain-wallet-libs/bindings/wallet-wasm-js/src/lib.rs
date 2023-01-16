//! JavaScript and TypeScript bindings for the Jormungandr wallet SDK.

pub use certificates::Certificate;
pub use certificates::{
    vote_cast::{Payload, VoteCast},
    vote_plan::VotePlanId,
};
use chain_impl_mockchain::account::SpendingCounter;
use chain_impl_mockchain::{
    certificate::VoteCast as VoteCastLib, fragment::Fragment as FragmentLib,
};
pub use fragment::{Fragment, FragmentId};
use wasm_bindgen::prelude::*;

mod certificates;
mod fragment;
mod utils;

#[wasm_bindgen]
pub struct VoteCastTxBuilder(wallet_core::TxBuilder<VoteCastLib>);

/// Encapsulates blockchain settings needed for some operations.
#[wasm_bindgen]
pub struct Settings(wallet_core::Settings);

#[wasm_bindgen]
impl Settings {
    pub fn from_json(json: String) -> Result<Settings, JsValue> {
        Ok(Settings(
            serde_json::from_str(&json).map_err(|e| JsValue::from(e.to_string()))?,
        ))
    }

    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.0).map_err(|e| JsValue::from(e.to_string()))
    }
}

#[wasm_bindgen]
impl VoteCastTxBuilder {
    /// Initializing of the VoteCastTxBuilder
    ///
    pub fn new(settings: Settings, vote_cast: VoteCast) -> VoteCastTxBuilder {
        Self(wallet_core::TxBuilder::new(settings.0, vote_cast.0))
    }

    /// First step of the VoteCast transaction building process
    ///
    /// The `account` parameter gives the Ed25519Extended private key
    /// of the account.
    pub fn build_tx(
        mut self,
        hex_account_id: String,
        counter: u32,
        lane: usize,
    ) -> Result<VoteCastTxBuilder, JsValue> {
        self.0 = self
            .0
            .build_tx(
                hex_account_id,
                SpendingCounter::new(lane, counter).map_err(|e| JsValue::from(e.to_string()))?,
            )
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(self)
    }

    pub fn sign_tx(mut self, hex_account: String) -> Result<VoteCastTxBuilder, JsValue> {
        self.0 = self
            .0
            .sign_tx(
                hex::decode(hex_account)
                    .map_err(|e| JsValue::from(e.to_string()))?
                    .as_slice(),
            )
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(self)
    }

    /// Finish step of building VoteCast fragment
    pub fn finalize_tx(self) -> Result<Fragment, JsValue> {
        self.0
            .finalize_tx((), FragmentLib::VoteCast)
            .map_err(|e| JsValue::from(e.to_string()))
            .map(Fragment)
    }
}
