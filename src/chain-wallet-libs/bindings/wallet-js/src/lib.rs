use wasm_bindgen::prelude::*;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Wallet(wallet_core::Wallet);

#[wasm_bindgen]
pub struct Settings(wallet_core::Settings);

#[wasm_bindgen]
pub struct Conversion(wallet_core::Conversion);

#[wasm_bindgen]
impl Wallet {
    /// retrieve a wallet from the given mnemonics and password
    ///
    /// this function will work for all yoroi, daedalus and other wallets
    /// as it will try every kind of wallet anyway
    ///
    /// You can also use this function to recover a wallet even after you have
    /// transferred all the funds to the new format (see the _convert_ function)
    ///
    /// the mnemonics should be in english
    pub fn recover(mnemonics: &str, password: &[u8]) -> Result<Wallet, JsValue> {
        wallet_core::Wallet::recover(mnemonics, password)
            .map_err(|e| JsValue::from(e.to_string()))
            .map(Wallet)
    }

    pub fn convert(&mut self, settings: &Settings) -> Conversion {
        Conversion(self.0.convert(settings.0.clone()))
    }

    /// retrieve funds from daedalus or yoroi wallet in the given block0 (or
    /// any other blocks).
    ///
    /// Execute this function then you can check who much funds you have
    /// retrieved from the given block.
    ///
    /// this function may take sometimes so it is better to only call this
    /// function if needed.
    ///
    /// also, this function should not be called twice with the same block.
    pub fn retrieve_funds(&mut self, block0: &[u8]) -> Result<Settings, JsValue> {
        self.0
            .retrieve_funds(block0)
            .map_err(|e| JsValue::from(e.to_string()))
            .map(Settings)
    }

    /// get the total value in the wallet
    ///
    /// make sure to call `retrieve_funds` prior to calling this function
    /// otherwise you will always have `0`
    pub fn total_value(&self) -> u64 {
        self.0.total_value().0
    }

    /// update the wallet account state
    ///
    /// this is the value retrieved from any jormungandr endpoint that allows to query
    /// for the account state. It gives the value associated to the account as well as
    /// the counter.
    ///
    /// It is important to be sure to have an updated wallet state before doing any
    /// transactions otherwise future transactions may fail to be accepted by any
    /// nodes of the blockchain because of invalid signature state.
    ///
    pub fn set_state(&mut self, value: u64, counter: u32) {
        self.0.set_state(wallet_core::Value(value), counter);
    }
}

#[wasm_bindgen]
impl Conversion {
    /// retrieve the total number of ignored UTxOs in the conversion
    /// transactions
    ///
    /// this is the number of utxos that are not included in the conversions
    /// because it is more expensive to use them than to ignore them. This is
    /// called dust.
    pub fn num_ignored(&self) -> usize {
        self.0.ignored().len()
    }

    /// retrieve the total value lost in dust utxos
    ///
    /// this is the total value of all the ignored UTxOs because
    /// they are too expensive to use in any transactions.
    ///
    /// I.e. their individual fee to add as an input is higher
    /// than the value they individually holds
    pub fn total_value_ignored(&self) -> u64 {
        self.0
            .ignored()
            .iter()
            .map(|i| *i.value().as_ref())
            .sum::<u64>()
    }

    /// the number of transactions built for the conversion
    pub fn transactions_len(&self) -> usize {
        self.0.transactions().len()
    }

    pub fn transactions_get(&self, index: usize) -> Option<Vec<u8>> {
        self.0.transactions().get(index).map(|t| t.to_owned())
    }
}
