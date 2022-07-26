//! Cryptography for Ethereum types.
pub mod secp256k1 {
    //! Re-export of types for constructing Ethereum signatures.
    pub use secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        Error, Message,
    };
}

pub mod sha3 {
    //! Re-export of types and traits for constructing Ethereum hashes used in signatures.
    pub use sha3::{Digest, Keccak256};
}
