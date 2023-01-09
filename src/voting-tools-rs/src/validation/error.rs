use cardano_serialization_lib::chain_crypto::{SignatureError, PublicKeyError};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidationError {
    #[error("failed to parse signature: {0}")]
    InvalidSignature(#[from] SignatureError),

    #[error("failed to parse public key: {0}")]
    InvalidPublicKey(#[from] PublicKeyError),
}
