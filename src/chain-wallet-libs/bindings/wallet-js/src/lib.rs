use js_sys::Array;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::convert::TryInto;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast as _;

mod utils;

const ENCRYPTION_VOTE_KEY_HRP: &str = "p256k1_votepk";

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

impl_secret_key!(
    Ed25519ExtendedPrivate,
    chain_crypto::Ed25519Extended,
    Ed25519Public
);
impl_secret_key!(Ed25519Private, chain_crypto::Ed25519, Ed25519Public);

impl_public_key!(Ed25519Public, chain_crypto::Ed25519);

#[wasm_bindgen]
pub struct Ed25519Signature(chain_crypto::Signature<Box<[u8]>, chain_crypto::Ed25519>);

#[wasm_bindgen]
pub struct FragmentId(wallet_core::FragmentId);

#[wasm_bindgen]
pub struct EncryptingVoteKey(chain_vote::EncryptingVoteKey);

/// this is used only for giving the Array a type in the typescript generated notation
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Array<FragmentId>")]
    pub type FragmentIds;
}

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

    pub fn import_keys(account: &[u8], keys: &[u8]) -> Result<Wallet, JsValue> {
        if keys.len() % 64 != 0 {
            return Err(JsValue::from_str("invalid keys array length"));
        }

        let keys: &[[u8; 64]] = unsafe {
            std::slice::from_raw_parts(keys.as_ptr().cast::<[u8; 64]>(), keys.len() / 64)
        };

        wallet_core::Wallet::recover_free_keys(account, keys)
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

    /// use this function to confirm a transaction has been properly received
    ///
    /// This function will automatically update the state of the wallet
    ///
    pub fn confirm_transaction(&mut self, fragment: &FragmentId) {
        self.0.confirm_transaction(fragment.0);
    }

    /// get the list of pending transaction ids, which can be used to query
    /// the status and then using `confirm_transaction` as needed.
    ///
    pub fn pending_transactions(&self) -> FragmentIds {
        self.0
            .pending_transactions()
            .iter()
            .cloned()
            .map(FragmentId)
            .map(JsValue::from)
            .collect::<Array>()
            .unchecked_into::<FragmentIds>()
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
    pub fn new_public(vote_plan_id: VotePlanId, index: u8, options: Options) -> Self {
        Proposal(wallet_core::Proposal::new(
            vote_plan_id.0.into(),
            index,
            options.0,
            wallet_core::PayloadTypeConfig::Public,
        ))
    }

    pub fn new_private(
        vote_plan_id: VotePlanId,
        index: u8,
        options: Options,
        encrypting_vote_key: EncryptingVoteKey,
    ) -> Self {
        Proposal(wallet_core::Proposal::new_private(
            vote_plan_id.0.into(),
            index,
            options.0,
            encrypting_vote_key.0,
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
impl Ed25519Signature {
    pub fn from_binary(signature: &[u8]) -> Result<Ed25519Signature, JsValue> {
        chain_crypto::Signature::from_binary(signature)
            .map(Self)
            .map_err(|e| JsValue::from_str(&format!("Invalid signature {}", e)))
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        self.0.as_ref().into()
    }
}

#[macro_export]
macro_rules! impl_public_key {
    ($name:ident, $wrapped_type:ty) => {
        #[wasm_bindgen]
        pub struct $name(chain_crypto::PublicKey<$wrapped_type>);

        #[wasm_bindgen]
        impl $name {
            pub fn bytes(&self) -> Box<[u8]> {
                self.0.as_ref().into()
            }

            pub fn bech32(&self) -> String {
                use chain_crypto::bech32::Bech32 as _;
                self.0.to_bech32_str()
            }

            pub fn verify(&self, signature: &Ed25519Signature, msg: &[u8]) -> bool {
                let verification = signature.0.verify_slice(&self.0, msg);
                match verification {
                    chain_crypto::Verification::Success => true,
                    chain_crypto::Verification::Failed => false,
                }
            }
        }
    };
}

/// macro arguments:
///     the exported name of the type
///     the inner/mangled key type
///     the name of the exported public key associated type
#[macro_export]
macro_rules! impl_secret_key {
    ($name:ident, $wrapped_type:ty, $public:ident) => {
        #[wasm_bindgen]
        pub struct $name(chain_crypto::SecretKey<$wrapped_type>);

        #[wasm_bindgen]
        impl $name {
            pub fn generate() -> $name {
                Self(chain_crypto::SecretKey::<$wrapped_type>::generate(
                    rand::rngs::OsRng,
                ))
            }

            /// optional seed to generate the key, for the same entropy the same key will be generated (32
            /// bytes). This seed will be fed to ChaChaRNG and allow pseudo random key
            /// generation. Do not use if you are not sure
            pub fn from_seed(seed: &[u8]) -> Result<$name, JsValue> {
                let seed: [u8; 32] = seed
                    .try_into()
                    .map_err(|_| JsValue::from_str("Invalid seed, expected 32 bytes"))?;

                let rng = ChaCha20Rng::from_seed(seed);

                Ok(Self(chain_crypto::SecretKey::<$wrapped_type>::generate(
                    rng,
                )))
            }

            pub fn public(&self) -> $public {
                $public(self.0.to_public())
            }

            pub fn bytes(&self) -> Box<[u8]> {
                self.0.clone().leak_secret().as_ref().into()
            }

            pub fn sign(&self, msg: &[u8]) -> Ed25519Signature {
                Ed25519Signature::from_binary(self.0.sign(&msg).as_ref()).unwrap()
            }
        }
    };
}

#[wasm_bindgen]
impl FragmentId {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<FragmentId, JsValue> {
        let array: [u8; std::mem::size_of::<wallet_core::FragmentId>()] = bytes
            .try_into()
            .map_err(|_| JsValue::from_str("Invalid fragment id"))?;

        Ok(FragmentId(array.into()))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

#[wasm_bindgen]
impl EncryptingVoteKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<EncryptingVoteKey, JsValue> {
        chain_vote::EncryptingVoteKey::from_bytes(&bytes)
            .ok_or_else(|| JsValue::from_str("invalid binary format"))
            .map(Self)
    }

    pub fn from_bech32(bech32_str: &str) -> Result<EncryptingVoteKey, JsValue> {
        use bech32::FromBase32;

        bech32::decode(bech32_str)
            .map_err(|e| JsValue::from_str(&format!("invalid bech32 string {}", e)))
            .and_then(|(hrp, raw_key)| {
                if hrp != ENCRYPTION_VOTE_KEY_HRP {
                    return Err(JsValue::from_str(&format!(
                        "expected hrp to be {} instead found {}",
                        ENCRYPTION_VOTE_KEY_HRP, hrp
                    )));
                }

                let bytes = Vec::<u8>::from_base32(&raw_key).unwrap();

                Self::from_bytes(&bytes)
            })
    }
}
