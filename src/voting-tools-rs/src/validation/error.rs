use cardano_serialization_lib::chain_crypto::{SignatureError, PublicKeyError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("failed to parse json: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("invalid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("failed to parse signature: {0}")]
    InvalidSignature(#[from] SignatureError),

    #[error("failed to parse public key: {0}")]
    InvalidPublicKey(#[from] PublicKeyError),
}
