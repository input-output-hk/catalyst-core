//! JavaScript and TypeScript bindings for the Jormungandr wallet SDK.

pub use certificates::Certificate;
pub use certificates::{
    vote_cast::{Payload, VoteCast},
    vote_plan::VotePlanId,
};
pub use fragment::{Fragment, FragmentId};
pub use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

mod certificates;
mod fragment;
mod utils;

/// A Wallet gives the user control over an account address
/// controlled by a private key. It can also be used to convert other funds
/// minted as UTxOs in the genesis block.
#[wasm_bindgen]
pub struct Wallet(wallet_core::Wallet);

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
pub struct BlockDate(chain_impl_mockchain::block::BlockDate);

#[wasm_bindgen]
impl BlockDate {
    pub fn new(epoch: u32, slot_id: u32) -> BlockDate {
        BlockDate(chain_impl_mockchain::block::BlockDate { epoch, slot_id })
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct SpendingCounter(chain_impl_mockchain::account::SpendingCounter);

#[wasm_bindgen]
impl SpendingCounter {
    pub fn new(lane: usize, counter: u32) -> Self {
        Self(chain_impl_mockchain::account::SpendingCounter::new(
            lane, counter,
        ))
    }
}

impl_collection!(SpendingCounters, SpendingCounter);

#[wasm_bindgen]
impl Wallet {
    /// Imports private key to create a wallet.
    ///
    /// The `account` parameter gives the Ed25519Extended private key
    /// of the account.
    pub fn import_key(account: &[u8]) -> Result<Wallet, JsValue> {
        wallet_core::Wallet::recover_free_keys(account)
            .map_err(|e| JsValue::from(e.to_string()))
            .map(Wallet)
    }

    /// get the account ID bytes
    ///
    /// This ID is also the account public key, it can be used to retrieve the
    /// account state (the value, transaction counter etc...).
    pub fn id(&self) -> Vec<u8> {
        self.0.id().as_ref().to_vec()
    }

    /// Get the total value in the wallet.
    ///
    /// Make sure to call `retrieve_funds` prior to calling this function,
    /// otherwise the function will return `0`.
    pub fn total_value(&self) -> u64 {
        self.0.total_value().0
    }

    /// Update the wallet account state.
    ///
    /// The values to update the account state with can be retrieved from a
    /// node API endpoint. It sets the balance value on the account
    /// as well as the current spending counter.
    ///
    /// It is important to be sure to have an up to date wallet state
    /// before doing any transactions, otherwise future transactions may fail
    /// to be accepted by the blockchain nodes because of an invalid witness
    /// signature.
    pub fn set_state(&mut self, value: u64, counters: SpendingCounters) -> Result<(), JsValue> {
        self.0
            .set_state(
                wallet_core::Value(value),
                counters.0.into_iter().map(|c| c.0.into()).collect(),
            )
            .map_err(|e| JsValue::from(e.to_string()))
    }

    pub fn sign_transaction(
        &mut self,
        settings: &Settings,
        valid_until: BlockDate,
        lane: u8,
        certificate: Certificate,
    ) -> Result<Fragment, JsValue> {
        let fragment = self
            .0
            .sign_transaction(&settings.0, valid_until.0, lane, certificate.0)
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(Fragment(fragment))
    }

    /// Confirms that a transaction has been confirmed on the blockchain.
    ///
    /// This function will update the state of the wallet tracking pending
    /// transactions on fund conversion.
    pub fn confirm_transaction(&mut self, fragment: &FragmentId) {
        self.0.confirm_transaction(fragment.0);
    }
}
