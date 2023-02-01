//! JavaScript and TypeScript bindings for the Jormungandr wallet SDK.

pub use certificates::Certificate;
pub use certificates::{
    vote_cast::{ElectionPublicKey, Payload, VoteCast},
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
    pub fn prepare_tx(mut self, hex_account_id: String) -> Result<VoteCastTxBuilder, JsValue> {
        self.0 = self
            .0
            .prepare_tx(hex_account_id, SpendingCounter::zero())
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(self)
    }

    /// Get a transaction signing data
    pub fn get_sign_data(&self) -> Result<Box<[u8]>, JsValue> {
        self.0
            .get_sign_data()
            .map(|data| data.as_ref().into())
            .map_err(|e| JsValue::from(e.to_string()))
    }

    /// Finish step of building VoteCast fragment with passing an already signed transaction data
    pub fn build_tx(self, hex_signature: String) -> Result<Fragment, JsValue> {
        self.0
            .build_tx(
                (),
                hex::decode(hex_signature)
                    .map_err(|e| JsValue::from(e.to_string()))?
                    .as_slice(),
                FragmentLib::VoteCast,
            )
            .map_err(|e| JsValue::from(e.to_string()))
            .map(Fragment)
    }

    /// Finish step of signing and building VoteCast fragment
    pub fn sign_tx(self, hex_account: String) -> Result<Fragment, JsValue> {
        self.0
            .sign_tx(
                (),
                hex::decode(hex_account)
                    .map_err(|e| JsValue::from(e.to_string()))?
                    .as_slice(),
                FragmentLib::VoteCast,
            )
            .map_err(|e| JsValue::from(e.to_string()))
            .map(Fragment)
    }
}
