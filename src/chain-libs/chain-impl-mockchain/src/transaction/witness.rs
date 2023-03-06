use super::element::TransactionSignDataHash;
use crate::account;
use crate::chaintypes::HeaderId;
use crate::key::{
    deserialize_public_key, deserialize_signature, serialize_public_key, serialize_signature,
    SpendingSignature,
};
use crate::multisig;
use chain_core::{
    packer::Codec,
    property::{Deserialize, DeserializeFromSlice, ReadError, Serialize, WriteError},
};
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
    Account(account::SpendingCounter, account::Witness),
    OldUtxo(
        PublicKey<Ed25519>,
        [u8; 32],
        Signature<WitnessUtxoData, Ed25519>,
    ),
    Multisig(account::SpendingCounter, multisig::Witness),
}

impl PartialEq for Witness {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Witness::Utxo(s1), Witness::Utxo(s2)) => s1.as_ref() == s2.as_ref(),
            (Witness::Account(n1, s1), Witness::Account(n2, s2)) => {
                n1 == n2 && s1.as_ref() == s2.as_ref()
            }
            (Witness::Multisig(n1, s1), Witness::Multisig(n2, s2)) => n1 == n2 && s1 == s2,
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
            Witness::Account(_, _) => write!(f, "Account Witness"),
            Witness::OldUtxo(..) => write!(f, "Old UTxO Witness"),
            Witness::Multisig(_, _) => write!(f, "Multisig Witness"),
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

    pub fn new_utxo_data(
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
    ) -> WitnessUtxoData {
        WitnessUtxoData::new(block0, sign_data_hash, WitnessUtxoVersion::Normal)
    }

    pub fn new_utxo<F>(block0: &HeaderId, sign_data_hash: &TransactionSignDataHash, sign: F) -> Self
    where
        F: FnOnce(&WitnessUtxoData) -> Signature<WitnessUtxoData, Ed25519>,
    {
        let wud = WitnessUtxoData::new(block0, sign_data_hash, WitnessUtxoVersion::Normal);
        Witness::Utxo(sign(&wud))
    }

    pub fn new_old_utxo_data(
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
    ) -> WitnessUtxoData {
        WitnessUtxoData::new(block0, sign_data_hash, WitnessUtxoVersion::Legacy)
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

    pub fn new_account_data(
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
        spending_counter: account::SpendingCounter,
    ) -> WitnessAccountData {
        WitnessAccountData::new(block0, sign_data_hash, spending_counter)
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
        Witness::Account(spending_counter, sig)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.serialize_as_vec()
            .expect("memory serialize is expected to just work")
    }
}

const WITNESS_TAG_OLDUTXO: u8 = 0u8;
const WITNESS_TAG_UTXO: u8 = 1u8;
const WITNESS_TAG_ACCOUNT: u8 = 2u8;
const WITNESS_TAG_MULTISIG: u8 = 3u8;

impl Serialize for Witness {
    fn serialized_size(&self) -> usize {
        match self {
            Witness::OldUtxo(pk, cc, sig) => {
                Codec::u8_size() + pk.as_ref().len() + cc.serialized_size() + sig.as_ref().len()
            }
            Witness::Utxo(sig) => Codec::u8_size() + sig.as_ref().len(),
            Witness::Account(_, sig) => Codec::u8_size() + Codec::u32_size() + sig.as_ref().len(),
            Witness::Multisig(_, msig) => {
                Codec::u8_size() + Codec::u32_size() + msig.serialized_size()
            }
        }
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        match self {
            Witness::OldUtxo(pk, cc, sig) => {
                codec.put_u8(WITNESS_TAG_OLDUTXO)?;
                serialize_public_key(pk, codec)?;
                codec.put_bytes(cc)?;
                serialize_signature(sig, codec)
            }
            Witness::Utxo(sig) => {
                codec.put_u8(WITNESS_TAG_UTXO)?;
                serialize_signature(sig, codec)
            }
            Witness::Account(nonce, sig) => {
                codec.put_u8(WITNESS_TAG_ACCOUNT)?;
                codec.put_be_u32((*nonce).into())?;
                serialize_signature(sig, codec)
            }
            Witness::Multisig(nonce, msig) => {
                codec.put_u8(WITNESS_TAG_MULTISIG)?;
                codec.put_be_u32((*nonce).into())?;
                msig.serialize(codec)
            }
        }
    }
}

impl DeserializeFromSlice for Witness {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        match codec.get_u8()? {
            WITNESS_TAG_OLDUTXO => {
                let pk = deserialize_public_key(codec)?;
                let some_bytes = <[u8; 32]>::deserialize(codec)?;
                let sig = deserialize_signature(codec)?;
                Ok(Witness::OldUtxo(pk, some_bytes, sig))
            }
            WITNESS_TAG_UTXO => deserialize_signature(codec).map(Witness::Utxo),
            WITNESS_TAG_ACCOUNT => {
                let nonce = codec.get_be_u32()?.into();
                let sig = deserialize_signature(codec)?;
                Ok(Witness::Account(nonce, sig))
            }
            WITNESS_TAG_MULTISIG => {
                let nonce = codec.get_be_u32()?.into();
                let msig = multisig::Witness::deserialize_from_slice(codec)?;
                Ok(Witness::Multisig(nonce, msig))
            }
            i => Err(ReadError::UnknownTag(i as u32)),
        }
    }
}
