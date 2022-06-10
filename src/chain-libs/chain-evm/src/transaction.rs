use crate::{util::Secret, Address};
use ethereum::{
    EIP1559TransactionMessage, EIP2930TransactionMessage, LegacyTransactionMessage,
    TransactionSignature, TransactionV2,
};
use ethereum_types::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message,
};
use sha3::{Digest, Keccak256};

/// Byte size for 'r' and 's' components of a signature.
const SIGNATURE_BYTES: usize = 32;

/// Wrapper type for `ethereum::TransactionV2`, which includes the `EIP1559Transaction`, `EIP2930Transaction`, `LegacyTransaction` variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EthereumUnsignedTransaction {
    Legacy(LegacyTransactionMessage),
    EIP2930(EIP2930TransactionMessage),
    EIP1559(EIP1559TransactionMessage),
}

impl EthereumUnsignedTransaction {
    pub fn hash(&self) -> H256 {
        match self {
            Self::Legacy(tx) => tx.hash(),
            Self::EIP2930(tx) => tx.hash(),
            Self::EIP1559(tx) => tx.hash(),
        }
    }

    /// Sign the current transaction given an H256-encoded secret key.
    ///
    /// Legacy transaction signature as specified in [EIP-155](https://eips.ethereum.org/EIPS/eip-155).
    pub fn sign(self, secret: &Secret) -> Result<EthereumSignedTransaction, secp256k1::Error> {
        match self {
            Self::Legacy(tx) => {
                let sig = super::util::sign_data_hash(&tx.hash(), secret)?;
                let (recovery_id, sig_bytes) = sig.serialize_compact();
                let v = if let Some(chain_id) = tx.chain_id {
                    recovery_id.to_i32() as u64 + chain_id * 2 + 35
                } else {
                    recovery_id.to_i32() as u64 + 27
                };
                let (r, s) = sig_bytes.split_at(SIGNATURE_BYTES);
                let signature =
                    TransactionSignature::new(v, H256::from_slice(r), H256::from_slice(s))
                        .ok_or(secp256k1::Error::InvalidSignature)?;
                Ok(EthereumSignedTransaction(TransactionV2::Legacy(
                    ethereum::LegacyTransaction {
                        nonce: tx.nonce,
                        gas_price: tx.gas_price,
                        gas_limit: tx.gas_limit,
                        action: tx.action,
                        value: tx.value,
                        input: tx.input,
                        signature,
                    },
                )))
            }
            Self::EIP2930(tx) => {
                let sig = super::util::sign_data_hash(&tx.hash(), secret)?;
                let (recovery_id, sig_bytes) = sig.serialize_compact();
                let (r, s) = sig_bytes.split_at(SIGNATURE_BYTES);
                let signature = TransactionSignature::new(
                    recovery_id.to_i32() as u64,
                    H256::from_slice(r),
                    H256::from_slice(s),
                )
                .ok_or(secp256k1::Error::InvalidSignature)?;
                Ok(EthereumSignedTransaction(TransactionV2::EIP2930(
                    ethereum::EIP2930Transaction {
                        chain_id: tx.chain_id,
                        nonce: tx.nonce,
                        gas_price: tx.gas_price,
                        gas_limit: tx.gas_limit,
                        action: tx.action,
                        value: tx.value,
                        input: tx.input,
                        access_list: tx.access_list,
                        odd_y_parity: recovery_id.to_i32() != 0,
                        r: *signature.r(),
                        s: *signature.s(),
                    },
                )))
            }
            Self::EIP1559(tx) => {
                let sig = super::util::sign_data_hash(&tx.hash(), secret)?;
                let (recovery_id, sig_bytes) = sig.serialize_compact();
                let (r, s) = sig_bytes.split_at(SIGNATURE_BYTES);
                let signature = TransactionSignature::new(
                    recovery_id.to_i32() as u64,
                    H256::from_slice(r),
                    H256::from_slice(s),
                )
                .ok_or(secp256k1::Error::InvalidSignature)?;
                Ok(EthereumSignedTransaction(TransactionV2::EIP1559(
                    ethereum::EIP1559Transaction {
                        chain_id: tx.chain_id,
                        nonce: tx.nonce,
                        max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                        max_fee_per_gas: tx.max_fee_per_gas,
                        gas_limit: tx.gas_limit,
                        action: tx.action,
                        value: tx.value,
                        input: tx.input,
                        access_list: tx.access_list,
                        odd_y_parity: recovery_id.to_i32() != 0,
                        r: *signature.r(),
                        s: *signature.s(),
                    },
                )))
            }
        }
    }
}

/// Wrapper type for `ethereum::TransactionV2`, which includes the `EIP1559Transaction`, `EIP2930Transaction`, `LegacyTransaction` variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EthereumSignedTransaction(pub TransactionV2);

impl EthereumSignedTransaction {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.rlp_bytes().freeze().to_vec()
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, DecoderError> {
        let rlp = Rlp::new(data);
        EthereumSignedTransaction::decode(&rlp)
    }

    pub fn recover(&self) -> Result<Address, secp256k1::Error> {
        match &self.0 {
            TransactionV2::Legacy(tx) => {
                let signature = tx.signature.clone();
                let recid = RecoveryId::from_i32(signature.standard_v() as i32)?;
                let data = {
                    let r = signature.r().as_fixed_bytes();
                    let s = signature.s().as_fixed_bytes();
                    let mut data = [0u8; 64];
                    data[..SIGNATURE_BYTES].copy_from_slice(&r[..]);
                    data[SIGNATURE_BYTES..].copy_from_slice(&s[..]);
                    data
                };
                let signature = RecoverableSignature::from_compact(&data, recid)?;
                let tx_hash = LegacyTransactionMessage::from(tx.clone()).hash();
                let msg = Message::from_slice(tx_hash.as_fixed_bytes())?;
                let pubkey = signature.recover(&msg)?;
                let pubkey_bytes = pubkey.serialize_uncompressed();
                Ok(Address::from_slice(
                    &Keccak256::digest(&pubkey_bytes[1..]).as_slice()[12..],
                ))
            }
            TransactionV2::EIP2930(tx) => {
                let recid = RecoveryId::from_i32(tx.odd_y_parity as i32)?;
                let data = {
                    let r = tx.r.as_fixed_bytes();
                    let s = tx.s.as_fixed_bytes();
                    let mut data = [0u8; 64];
                    data[..SIGNATURE_BYTES].copy_from_slice(&r[..]);
                    data[SIGNATURE_BYTES..].copy_from_slice(&s[..]);
                    data
                };
                let signature = RecoverableSignature::from_compact(&data, recid)?;
                let tx_hash = EIP2930TransactionMessage::from(tx.clone()).hash();
                let msg = Message::from_slice(tx_hash.as_fixed_bytes())?;
                let pubkey = signature.recover(&msg)?;
                let pubkey_bytes = pubkey.serialize_uncompressed();
                Ok(Address::from_slice(
                    &Keccak256::digest(&pubkey_bytes[1..]).as_slice()[12..],
                ))
            }
            TransactionV2::EIP1559(tx) => {
                let recid = RecoveryId::from_i32(tx.odd_y_parity as i32)?;
                let data = {
                    let r = tx.r.as_fixed_bytes();
                    let s = tx.s.as_fixed_bytes();
                    let mut data = [0u8; 64];
                    data[..SIGNATURE_BYTES].copy_from_slice(&r[..]);
                    data[SIGNATURE_BYTES..].copy_from_slice(&s[..]);
                    data
                };
                let signature = RecoverableSignature::from_compact(&data, recid)?;
                let tx_hash = EIP1559TransactionMessage::from(tx.clone()).hash();
                let msg = Message::from_slice(tx_hash.as_fixed_bytes())?;
                let pubkey = signature.recover(&msg)?;
                let pubkey_bytes = pubkey.serialize_uncompressed();
                Ok(Address::from_slice(
                    &Keccak256::digest(&pubkey_bytes[1..]).as_slice()[12..],
                ))
            }
        }
    }
}

impl From<EthereumSignedTransaction> for EthereumUnsignedTransaction {
    fn from(other: EthereumSignedTransaction) -> Self {
        match other.0 {
            TransactionV2::Legacy(tx) => {
                EthereumUnsignedTransaction::Legacy(LegacyTransactionMessage::from(tx))
            }
            TransactionV2::EIP2930(tx) => {
                EthereumUnsignedTransaction::EIP2930(EIP2930TransactionMessage::from(tx))
            }
            TransactionV2::EIP1559(tx) => {
                EthereumUnsignedTransaction::EIP1559(EIP1559TransactionMessage::from(tx))
            }
        }
    }
}

impl Decodable for EthereumSignedTransaction {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let tx = TransactionV2::decode(rlp)?;
        Ok(EthereumSignedTransaction(tx))
    }
}

impl Encodable for EthereumSignedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.0.rlp_append(s);
    }
}
