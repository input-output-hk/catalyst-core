use super::element::TransactionSignDataHash;
use crate::account;
use crate::chaintypes::HeaderId;
use crate::key::{
    deserialize_public_key, deserialize_signature, serialize_public_key, serialize_signature,
    SpendingSignature,
};
use crate::multisig;
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;
use chain_crypto::{Ed25519, PublicKey, Signature};

/// Structure that proofs that certain user agrees with
/// some data. This structure is used to sign `Transaction`
/// and get `SignedTransaction` out.
///
/// It's important that witness works with opaque structures
/// and may not know the contents of the internal transaction.
#[derive(Debug, Clone)]
pub enum Witness {
    Utxo(SpendingSignature<WitnessUtxoData>),
    Account(account::Witness),
    OldUtxo(
        PublicKey<Ed25519>,
        [u8; 32],
        Signature<WitnessUtxoData, Ed25519>,
    ),
    Multisig(multisig::Witness),
}

impl PartialEq for Witness {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Witness::Utxo(s1), Witness::Utxo(s2)) => s1.as_ref() == s2.as_ref(),
            (Witness::Account(s1), Witness::Account(s2)) => s1.as_ref() == s2.as_ref(),
            (Witness::Multisig(s1), Witness::Multisig(s2)) => s1 == s2,
            (Witness::OldUtxo(p1, c1, s1), Witness::OldUtxo(p2, c2, s2)) => {
                s1.as_ref() == s2.as_ref() && c1 == c2 && p1 == p2
            }
            (_, _) => false,
        }
    }
}
impl Eq for Witness {}

impl std::fmt::Display for Witness {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Witness::Utxo(_) => write!(f, "UTxO Witness"),
            Witness::Account(_) => write!(f, "Account Witness"),
            Witness::OldUtxo(..) => write!(f, "Old UTxO Witness"),
            Witness::Multisig(_) => write!(f, "Multisig Witness"),
        }
    }
}

pub struct WitnessUtxoData(Vec<u8>);

#[derive(Debug, Clone, Copy)]
pub enum WitnessUtxoVersion {
    Legacy,
    Normal,
}

fn witness_data_common(
    data: &mut Vec<u8>,
    tag: u8,
    block0: &HeaderId,
    transaction_id: &TransactionSignDataHash,
) {
    data.push(tag);
    data.extend_from_slice(block0.as_ref());
    data.extend_from_slice(transaction_id.as_ref());
}

impl WitnessUtxoData {
    pub fn new(
        block0: &HeaderId,
        transaction_id: &TransactionSignDataHash,
        utxo_version: WitnessUtxoVersion,
    ) -> Self {
        let mut v = Vec::with_capacity(65);
        let tag = match utxo_version {
            WitnessUtxoVersion::Legacy => WITNESS_TAG_OLDUTXO,
            WitnessUtxoVersion::Normal => WITNESS_TAG_UTXO,
        };
        witness_data_common(&mut v, tag, block0, transaction_id);
        WitnessUtxoData(v)
    }
}

impl AsRef<[u8]> for WitnessUtxoData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub struct WitnessAccountData(Vec<u8>);

impl WitnessAccountData {
    pub fn new(
        block0: &HeaderId,
        transaction_id: &TransactionSignDataHash,
        spending_counter: account::SpendingCounter,
    ) -> Self {
        let mut v = Vec::with_capacity(69);
        witness_data_common(&mut v, WITNESS_TAG_ACCOUNT, block0, transaction_id);
        v.extend_from_slice(&spending_counter.to_bytes());
        WitnessAccountData(v)
    }
}

impl AsRef<[u8]> for WitnessAccountData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub struct WitnessMultisigData(Vec<u8>);

impl WitnessMultisigData {
    pub fn new(
        block0: &HeaderId,
        transaction_id: &TransactionSignDataHash,
        spending_counter: account::SpendingCounter,
    ) -> Self {
        let mut v = Vec::with_capacity(69);
        witness_data_common(&mut v, WITNESS_TAG_MULTISIG, block0, transaction_id);
        v.extend_from_slice(&spending_counter.to_bytes());
        Self(v)
    }
}

impl AsRef<[u8]> for WitnessMultisigData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Witness {
    /// Creates new `Witness` value.
    pub fn new_utxo<F>(block0: &HeaderId, sign_data_hash: &TransactionSignDataHash, sign: F) -> Self
    where
        F: FnOnce(&WitnessUtxoData) -> Signature<WitnessUtxoData, Ed25519>,
    {
        let wud = WitnessUtxoData::new(block0, sign_data_hash, WitnessUtxoVersion::Normal);
        Witness::Utxo(sign(&wud))
    }

    pub fn new_old_utxo<F>(
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
        sign: F,
        some_bytes: &[u8; 32],
    ) -> Self
    where
        F: FnOnce(&WitnessUtxoData) -> (PublicKey<Ed25519>, Signature<WitnessUtxoData, Ed25519>),
    {
        let wud = WitnessUtxoData::new(block0, sign_data_hash, WitnessUtxoVersion::Legacy);
        let (pk, sig) = sign(&wud);
        Witness::OldUtxo(pk, *some_bytes, sig)
    }

    pub fn new_account<F>(
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
        spending_counter: account::SpendingCounter,
        sign: F,
    ) -> Self
    where
        F: FnOnce(&WitnessAccountData) -> account::Witness,
    {
        let wud = WitnessAccountData::new(block0, sign_data_hash, spending_counter);
        let sig = sign(&wud);
        Witness::Account(sig)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        use chain_core::property::Serialize;
        self.serialize_as_vec()
            .expect("memory serialize is expected to just work")
    }
}

const WITNESS_TAG_OLDUTXO: u8 = 0u8;
const WITNESS_TAG_UTXO: u8 = 1u8;
const WITNESS_TAG_ACCOUNT: u8 = 2u8;
const WITNESS_TAG_MULTISIG: u8 = 3u8;

impl property::Serialize for Witness {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        use std::io::Write;

        let mut codec = Codec::new(writer);
        match self {
            Witness::OldUtxo(pk, cc, sig) => {
                codec.put_u8(WITNESS_TAG_OLDUTXO)?;
                serialize_public_key(pk, &mut codec)?;
                codec.write_all(cc)?;
                serialize_signature(sig, &mut codec)
            }
            Witness::Utxo(sig) => {
                codec.put_u8(WITNESS_TAG_UTXO)?;
                serialize_signature(sig, codec.into_inner())
            }
            Witness::Account(sig) => {
                codec.put_u8(WITNESS_TAG_ACCOUNT)?;
                serialize_signature(sig, codec.into_inner())
            }
            Witness::Multisig(msig) => {
                codec.put_u8(WITNESS_TAG_MULTISIG)?;
                msig.serialize(codec.into_inner())
            }
        }
    }
}

impl Readable for Witness {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        match buf.get_u8()? {
            WITNESS_TAG_OLDUTXO => {
                let pk = deserialize_public_key(buf)?;
                let some_bytes = <[u8; 32]>::read(buf)?;
                let sig = deserialize_signature(buf)?;
                Ok(Witness::OldUtxo(pk, some_bytes, sig))
            }
            WITNESS_TAG_UTXO => deserialize_signature(buf).map(Witness::Utxo),
            WITNESS_TAG_ACCOUNT => deserialize_signature(buf).map(Witness::Account),
            WITNESS_TAG_MULTISIG => {
                let msig = multisig::Witness::read(buf)?;
                Ok(Witness::Multisig(msig))
            }
            i => Err(ReadError::UnknownTag(i as u32)),
        }
    }
}
