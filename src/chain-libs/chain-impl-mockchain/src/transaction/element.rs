use crate::key::deserialize_signature;
use crate::transaction::TransactionBindingAuthData;
use crate::value::{Value, ValueError};
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_crypto::{digest::DigestOf, Blake2b256, Ed25519, PublicKey, Signature, Verification};
use thiserror::Error;
use typed_bytes::ByteBuilder;

pub struct TransactionSignData(Box<[u8]>);

impl From<Vec<u8>> for TransactionSignData {
    fn from(v: Vec<u8>) -> TransactionSignData {
        TransactionSignData(v.into())
    }
}

impl AsRef<[u8]> for TransactionSignData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub type TransactionSignDataHash = DigestOf<Blake2b256, TransactionSignData>;

// cannot use TransactionSignDataHash<'a> as a phantom in the signature,
// as the lifetime is still kept behind, so invent a new phantom type
// to track the SignDataHash
#[derive(Debug, Clone)]
pub struct TransactionBindingAuthDataPhantom();

#[derive(Debug, Clone)]
pub struct SingleAccountBindingSignature(
    pub(crate) Signature<TransactionBindingAuthDataPhantom, Ed25519>,
);

impl SingleAccountBindingSignature {
    pub fn verify_slice<'a>(
        &self,
        pk: &PublicKey<Ed25519>,
        data: &TransactionBindingAuthData<'a>,
    ) -> Verification {
        self.0.verify_slice(pk, data.0)
    }

    pub fn new<'a, F>(data: &TransactionBindingAuthData<'a>, sign: F) -> Self
    where
        F: FnOnce(
            &TransactionBindingAuthData<'a>,
        ) -> Signature<TransactionBindingAuthDataPhantom, Ed25519>,
    {
        let sig = sign(data);
        SingleAccountBindingSignature(sig) // sk.sign_slice(data.0))
    }
}

impl AsRef<[u8]> for SingleAccountBindingSignature {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Readable for SingleAccountBindingSignature {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        deserialize_signature(buf).map(SingleAccountBindingSignature)
    }
}

#[derive(Debug, Clone)]
pub enum AccountBindingSignature {
    Single(SingleAccountBindingSignature),
    Multi(u32), // TODO
}

impl AccountBindingSignature {
    pub fn new_single<'a, F>(data: &TransactionBindingAuthData<'a>, sign: F) -> Self
    where
        F: FnOnce(
            &TransactionBindingAuthData<'a>,
        ) -> Signature<TransactionBindingAuthDataPhantom, Ed25519>,
    {
        AccountBindingSignature::Single(SingleAccountBindingSignature::new(data, sign))
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        match self {
            AccountBindingSignature::Single(sig) => bb.u8(1).bytes(sig.as_ref()),
            AccountBindingSignature::Multi(_) => {
                bb.u8(2);
                unimplemented!()
            }
        }
    }
}

impl Readable for AccountBindingSignature {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        match buf.get_u8()? {
            1 => {
                let sig = deserialize_signature(buf).map(SingleAccountBindingSignature)?;
                Ok(AccountBindingSignature::Single(sig))
            }
            2 => unimplemented!(),
            n => Err(ReadError::UnknownTag(n as u32)),
        }
    }
}

/// Amount of the balance in the transaction.
pub enum Balance {
    /// Balance is positive.
    Positive(Value),
    /// Balance is negative, such transaction can't be valid.
    Negative(Value),
    /// Balance is zero.
    Zero,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BalanceError {
    #[error("failed to compute total input")]
    InputsTotalFailed(#[source] ValueError),
    #[error("failed to compute total output")]
    OutputsTotalFailed(#[source] ValueError),
    #[error("transaction value not balanced, has inputs sum {inputs} and outputs sum {outputs}")]
    NotBalanced { inputs: Value, outputs: Value },
}
