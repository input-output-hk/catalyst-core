//! Ethereum signatures for EIP-191, Legacy Transactions (EIP-155), EIP-2930, and EIP-1559.
use crate::util::Secret;
use ethereum::{LegacyTransactionMessage, TransactionSignature};
use ethereum_types::H256;
use secp256k1::ecdsa::RecoverableSignature;

/// Byte size for 'r' and 's' components of a signature.
const SIGNATURE_BYTES: usize = 32;

/// Legacy transaction signature, as specified in [EIP-155](https://eips.ethereum.org/EIPS/eip-155).
pub fn sign_eip_155(
    tx: &LegacyTransactionMessage,
    secret: &Secret,
) -> Result<TransactionSignature, secp256k1::Error> {
    let sig = super::util::sign_data_hash(&tx.hash(), secret)?;
    let (recovery_id, sig_bytes) = sig.serialize_compact();
    let v = if let Some(chain_id) = tx.chain_id {
        recovery_id.to_i32() as u64 + chain_id * 2 + 35
    } else {
        recovery_id.to_i32() as u64 + 27
    };
    let (r, s) = sig_bytes.split_at(SIGNATURE_BYTES);
    TransactionSignature::new(v, H256::from_slice(r), H256::from_slice(s))
        .ok_or(secp256k1::Error::InvalidSignature)
}

/// Type 1 transaction signature, as specified in [EIP-2930](https://eips.ethereum.org/EIPS/eip-2930).
pub fn eip_2930_signature(
    tx_hash: &H256,
    secret: &Secret,
) -> Result<TransactionSignature, secp256k1::Error> {
    let sig = super::util::sign_data_hash(tx_hash, secret)?;
    let (recovery_id, sig_bytes) = sig.serialize_compact();
    let (r, s) = sig_bytes.split_at(SIGNATURE_BYTES);
    TransactionSignature::new(
        recovery_id.to_i32() as u64 % 2,
        H256::from_slice(r),
        H256::from_slice(s),
    )
    .ok_or(secp256k1::Error::InvalidSignature)
}

/// Type 2 transaction signature, as specified in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559).
pub fn eip_1559_signature(
    tx_hash: &H256,
    secret: &Secret,
) -> Result<TransactionSignature, secp256k1::Error> {
    eip_2930_signature(tx_hash, secret)
}

/// Signature for hex-encoded strings, as specified in [EIP-191](https://eips.ethereum.org/EIPS/eip-191).
pub fn eip_191_signature<T: AsRef<[u8]>>(
    data: T,
    secret: &Secret,
) -> Result<RecoverableSignature, secp256k1::Error> {
    // first we check if the message is a valid hex-encoded message
    let msg = hex::decode(data).map_err(|_| secp256k1::Error::InvalidSignature)?;
    let msg_for_hash = format!("\x19Ethereum Signed Message:\n{}{:?}", msg.len(), msg);
    super::util::sign_data(msg_for_hash.as_bytes(), secret)
}
