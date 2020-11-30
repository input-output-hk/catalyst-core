use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::convert::TryInto;

use wasm_bindgen::prelude::*;

mod utils;
pub use utils::set_panic_hook;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Ed25519ExtendedPrivate(chain_crypto::SecretKey<chain_crypto::Ed25519Extended>);

#[wasm_bindgen]
impl Ed25519ExtendedPrivate {
    pub fn generate() -> Ed25519ExtendedPrivate {
        Self(chain_crypto::SecretKey::<chain_crypto::Ed25519Extended>::generate(rand::rngs::OsRng))
    }

    /// optional seed to generate the key, for the same entropy the same key will be generated (32
    /// bytes). This seed will be fed to ChaChaRNG and allow pseudo random key
    /// generation. Do not use if you are not sure
    pub fn from_seed(seed: &[u8]) -> Result<Ed25519ExtendedPrivate, JsValue> {
        let seed: [u8; 32] = seed
            .try_into()
            .map_err(|_| JsValue::from_str("Invalid seed, expected 32 bytes"))?;

        let rng = ChaCha20Rng::from_seed(seed);

        Ok(Self(
            chain_crypto::SecretKey::<chain_crypto::Ed25519Extended>::generate(rng),
        ))
    }

    pub fn public(&self) -> Ed25519Public {
        Ed25519Public(self.0.to_public())
    }

    pub fn bytes(&self) -> Box<[u8]> {
        self.0.clone().leak_secret().as_ref().into()
    }
}

#[wasm_bindgen]
pub struct Ed25519Signature(chain_crypto::Signature<Box<[u8]>, chain_crypto::Ed25519>);

#[wasm_bindgen]
pub struct Ed25519Public(chain_crypto::PublicKey<chain_crypto::Ed25519>);

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
/// decode a bech32 string to a byte array, disregarding the hrp
pub fn bech32_decode_to_bytes(input: &str) -> Result<Vec<u8>, JsValue> {
    bech32::decode(input)
        .and_then(|(_hrp, words)| bech32::FromBase32::from_base32(&words))
        .map_err(|err| JsValue::from_str(&format!("{}", err)))
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
