use std::convert::TryInto;
use wasm_bindgen::prelude::*;

mod utils;

// `set_panic_hook` function can be called at least once during initialization,
// to get better error messages if the code ever panics.
pub use utils::set_panic_hook;

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
pub struct Proposal(wallet_core::Proposal);

#[wasm_bindgen]
pub struct VotePlanId([u8; wallet_core::VOTE_PLAN_ID_LENGTH]);

#[wasm_bindgen]
pub struct Options(wallet_core::Options);

#[wasm_bindgen]
pub enum PayloadType {
    Public,
}

#[wasm_bindgen]
pub struct Ed25519ExtendedPrivate(chain_crypto::SecretKey<chain_crypto::Ed25519Extended>);

#[wasm_bindgen]
pub struct Ed25519Public(chain_crypto::PublicKey<chain_crypto::Ed25519>);

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

    /// get the account ID bytes
    ///
    /// This ID is also the account public key, it can be used to retrieve the
    /// account state (the value, transaction counter etc...).
    pub fn id(&self) -> Vec<u8> {
        self.0.id().as_ref().to_vec()
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

    /// Cast a vote
    ///
    /// This function outputs a fragment containing a voting transaction.
    ///
    /// # Parameters
    ///
    /// * `settings` - ledger settings.
    /// * `proposal` - proposal information including the range of values
    ///   allowed in `choice`.
    /// * `choice` - the option to vote for.
    ///
    /// # Errors
    ///
    /// The error is returned when `choice` does not fall withing the range of
    /// available choices specified in `proposal`.
    pub fn vote(
        &mut self,
        settings: &Settings,
        proposal: &Proposal,
        choice: u8,
    ) -> Result<Box<[u8]>, JsValue> {
        self.0
            .vote(
                settings.0.clone(),
                &proposal.0,
                wallet_core::Choice::new(choice),
            )
            .map_err(|e| JsValue::from(e.to_string()))
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

#[wasm_bindgen]
impl Proposal {
    pub fn new(
        vote_plan_id: VotePlanId,
        payload_type: PayloadType,
        index: u8,
        options: Options,
    ) -> Self {
        let payload_type = match payload_type {
            PayloadType::Public => wallet_core::PayloadType::Public,
        };
        Proposal(wallet_core::Proposal::new(
            vote_plan_id.0.into(),
            payload_type,
            index,
            options.0,
        ))
    }
}

#[wasm_bindgen]
impl VotePlanId {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<VotePlanId, JsValue> {
        let array: [u8; wallet_core::VOTE_PLAN_ID_LENGTH] = bytes
            .try_into()
            .map_err(|_| JsValue::from_str("Invalid vote plan id length"))?;

        Ok(VotePlanId(array))
    }
}

#[wasm_bindgen]
impl Options {
    pub fn new_length(length: u8) -> Result<Options, JsValue> {
        wallet_core::Options::new_length(length)
            .map_err(|e| JsValue::from(e.to_string()))
            .map(Options)
    }
}

#[wasm_bindgen]
impl Ed25519ExtendedPrivate {
    pub fn generate() -> Self {
        Self(chain_crypto::SecretKey::<chain_crypto::Ed25519Extended>::generate(rand::rngs::OsRng))
    }

    pub fn public(&self) -> Ed25519Public {
        Ed25519Public(self.0.to_public())
    }

    pub fn bytes(&self) -> Box<[u8]> {
        self.0.clone().leak_secret().as_ref().into()
    }
}

#[wasm_bindgen]
impl Ed25519Public {
    pub fn bytes(&self) -> Box<[u8]> {
        self.0.as_ref().into()
    }

    pub fn bech32(&self) -> String {
        use chain_crypto::bech32::Bech32 as _;
        self.0.to_bech32_str()
    }
}

#[wasm_bindgen]
pub fn symmetric_encrypt(password: &[u8], data: &[u8]) -> Result<Box<[u8]>, JsValue> {
    symmetric_cipher::encrypt(password, data, rand::rngs::OsRng)
        .map_err(|e| JsValue::from_str(&format!("encryption failed {}", e)))
}

#[wasm_bindgen]
pub fn symmetric_decrypt(password: &[u8], data: &[u8]) -> Result<Box<[u8]>, JsValue> {
    symmetric_cipher::decrypt(password, data)
        .map_err(|e| JsValue::from_str(&format!("decryption failed {}", e)))
}
