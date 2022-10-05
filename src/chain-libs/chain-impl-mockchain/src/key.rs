//! Module provides cryptographic utilities and types related to
//! the user keys.
//!
use chain_core::{
    packer::Codec,
    property::{
        BlockId, Deserialize, DeserializeFromSlice, FragmentId, ReadError, Serialize, WriteError,
    },
};
use chain_crypto as crypto;
use chain_crypto::{
    digest::DigestOf, AsymmetricKey, AsymmetricPublicKey, Blake2b256, Ed25519, PublicKey,
    RistrettoGroup2HashDh, SecretKey, SigningAlgorithm, SumEd25519_12, VerificationAlgorithm,
};
use rand_core::{CryptoRng, RngCore};
use typed_bytes::ByteBuilder;

use std::str::FromStr;

#[derive(Clone)]
pub enum EitherEd25519SecretKey {
    Extended(crypto::SecretKey<crypto::Ed25519Extended>),
    Normal(crypto::SecretKey<crypto::Ed25519>),
}

impl EitherEd25519SecretKey {
    pub fn generate<R: RngCore + CryptoRng>(rng: R) -> Self {
        EitherEd25519SecretKey::Extended(SecretKey::generate(rng))
    }

    pub fn to_public(&self) -> crypto::PublicKey<crypto::Ed25519> {
        match self {
            EitherEd25519SecretKey::Extended(sk) => sk.to_public(),
            EitherEd25519SecretKey::Normal(sk) => sk.to_public(),
        }
    }

    pub fn sign<T: AsRef<[u8]>>(&self, dat: &T) -> crypto::Signature<T, crypto::Ed25519> {
        match self {
            EitherEd25519SecretKey::Extended(sk) => sk.sign(dat),
            EitherEd25519SecretKey::Normal(sk) => sk.sign(dat),
        }
    }

    pub fn sign_slice<T: ?Sized>(&self, dat: &[u8]) -> crypto::Signature<T, crypto::Ed25519> {
        match self {
            EitherEd25519SecretKey::Extended(sk) => sk.sign_slice(dat),
            EitherEd25519SecretKey::Normal(sk) => sk.sign_slice(dat),
        }
    }
}

pub type SpendingPublicKey = crypto::PublicKey<crypto::Ed25519>;
pub type SpendingSignature<T> = crypto::Signature<T, crypto::Ed25519>;

pub type AccountPublicKey = crypto::PublicKey<crypto::Ed25519>;
pub type AccountSignature<T> = crypto::Signature<T, crypto::Ed25519>;

pub type Ed25519Signature<T> = crypto::Signature<T, crypto::Ed25519>;

fn chain_crypto_pub_err(e: crypto::PublicKeyError) -> ReadError {
    match e {
        crypto::PublicKeyError::SizeInvalid => {
            ReadError::StructureInvalid("publickey size invalid".to_string())
        }
        crypto::PublicKeyError::StructureInvalid => {
            ReadError::StructureInvalid("publickey structure invalid".to_string())
        }
    }
}
fn chain_crypto_sig_err(e: crypto::SignatureError) -> ReadError {
    match e {
        crypto::SignatureError::SizeInvalid { expected, got } => ReadError::StructureInvalid(
            format!("signature size invalid, expected {} got {}", expected, got),
        ),
        crypto::SignatureError::StructureInvalid => {
            ReadError::StructureInvalid("signature structure invalid".to_string())
        }
    }
}

#[inline]
pub fn serialize_public_key<A: AsymmetricPublicKey, W: std::io::Write>(
    key: &crypto::PublicKey<A>,
    codec: &mut Codec<W>,
) -> Result<(), WriteError> {
    codec.put_bytes(key.as_ref())
}
#[inline]
pub fn serialize_signature<A: VerificationAlgorithm, T, W: std::io::Write>(
    signature: &crypto::Signature<T, A>,
    codec: &mut Codec<W>,
) -> Result<(), WriteError> {
    codec.put_bytes(signature.as_ref())
}
#[inline]
pub fn deserialize_public_key<A>(
    codec: &mut Codec<&[u8]>,
) -> Result<crypto::PublicKey<A>, ReadError>
where
    A: AsymmetricPublicKey,
{
    let bytes = codec.get_slice(A::PUBLIC_KEY_SIZE)?;
    crypto::PublicKey::from_binary(bytes).map_err(chain_crypto_pub_err)
}
#[inline]
pub fn deserialize_signature<A, T>(
    codec: &mut Codec<&[u8]>,
) -> Result<crypto::Signature<T, A>, ReadError>
where
    A: VerificationAlgorithm,
{
    let bytes = codec.get_slice(A::SIGNATURE_SIZE)?;
    crypto::Signature::from_binary(bytes).map_err(chain_crypto_sig_err)
}

pub fn make_signature<T, A>(
    spending_key: &crypto::SecretKey<A>,
    data: &T,
) -> crypto::Signature<T, A::PubAlg>
where
    A: SigningAlgorithm,
    <A as AsymmetricKey>::PubAlg: VerificationAlgorithm,
    T: Serialize,
{
    let bytes = data.serialize_as_vec().unwrap();
    spending_key.sign(&bytes).coerce()
}

pub fn verify_signature<T, A>(
    signature: &crypto::Signature<T, A>,
    public_key: &crypto::PublicKey<A>,
    data: &T,
) -> crypto::Verification
where
    A: VerificationAlgorithm,
    T: Serialize,
{
    let bytes = data.serialize_as_vec().unwrap();
    signature.clone().coerce().verify(public_key, &bytes)
}

pub fn verify_multi_signature<T, A>(
    signature: &crypto::Signature<T, A>,
    public_key: &[crypto::PublicKey<A>],
    data: &T,
) -> crypto::Verification
where
    A: VerificationAlgorithm,
    T: Serialize,
{
    assert!(!public_key.is_empty());
    let bytes = data.serialize_as_vec().unwrap();
    signature.clone().coerce().verify(&public_key[0], &bytes)
}

/// A serializable type T with a signature.
pub struct Signed<T, A: VerificationAlgorithm> {
    pub data: T,
    pub sig: crypto::Signature<T, A>,
}

pub fn signed_new<T: Serialize, A: SigningAlgorithm>(
    secret_key: &crypto::SecretKey<A>,
    data: T,
) -> Signed<T, A::PubAlg>
where
    A::PubAlg: VerificationAlgorithm,
{
    let bytes = data.serialize_as_vec().unwrap();
    let signature = secret_key.sign(&bytes).coerce();
    Signed {
        data,
        sig: signature,
    }
}

impl<T: Serialize, A: VerificationAlgorithm> Serialize for Signed<T, A> {
    fn serialized_size(&self) -> usize {
        self.data.serialized_size() + self.sig.as_ref().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        self.data.serialize(codec)?;
        serialize_signature(&self.sig, codec)
    }
}

impl<T: Deserialize, A: VerificationAlgorithm> DeserializeFromSlice for Signed<T, A> {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        Ok(Signed {
            data: T::deserialize(codec)?,
            sig: deserialize_signature(codec)?,
        })
    }
}

impl<T: PartialEq, A: VerificationAlgorithm> PartialEq<Self> for Signed<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data) && self.sig.as_ref() == other.sig.as_ref()
    }
}
impl<T: PartialEq, A: VerificationAlgorithm> Eq for Signed<T, A> {}
impl<T: std::fmt::Debug, A: VerificationAlgorithm> std::fmt::Debug for Signed<T, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Signed ( data: {:?}, signature: {:?} )",
            self.data,
            self.sig.as_ref()
        )
    }
}
impl<T: Clone, A: VerificationAlgorithm> Clone for Signed<T, A> {
    fn clone(&self) -> Self {
        Signed {
            data: self.data.clone(),
            sig: self.sig.clone(),
        }
    }
}

/// Hash that is used as an address of the various components.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct Hash(crypto::Blake2b256);
impl Hash {
    /// All 0 hash used as a special hash
    pub fn zero_hash() -> Self {
        Hash(crypto::Blake2b256::from([0; crypto::Blake2b256::HASH_SIZE]))
    }
    pub fn hash_bytes(bytes: &[u8]) -> Self {
        Hash(crypto::Blake2b256::new(bytes))
    }
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Hash(crypto::Blake2b256::from(bytes))
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; 32]> for Hash {
    fn from(a: [u8; 32]) -> Self {
        Hash::from_bytes(a)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(h: Hash) -> Self {
        h.0.into()
    }
}

impl<'a> From<&'a Hash> for &'a [u8; 32] {
    fn from(h: &'a Hash) -> Self {
        (&h.0).into()
    }
}

impl Serialize for Hash {
    fn serialized_size(&self) -> usize {
        self.0.as_hash_bytes().serialized_size()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.0.as_hash_bytes())
    }
}

impl Deserialize for Hash {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let bytes = <[u8; crypto::Blake2b256::HASH_SIZE]>::deserialize(codec)?;
        Ok(Hash(crypto::Blake2b256::from(bytes)))
    }
}

impl BlockId for Hash {
    fn zero() -> Hash {
        Hash(crypto::Blake2b256::from([0; crypto::Blake2b256::HASH_SIZE]))
    }
}

impl FragmentId for Hash {}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<crypto::Blake2b256> for Hash {
    fn from(hash: crypto::Blake2b256) -> Self {
        Hash(hash)
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Hash {
    type Err = crypto::hash::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Hash(crypto::Blake2b256::from_str(s)?))
    }
}

pub type BftVerificationAlg = Ed25519;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct BftLeaderId(pub(crate) PublicKey<BftVerificationAlg>);

impl From<[u8; 32]> for BftLeaderId {
    fn from(v: [u8; 32]) -> BftLeaderId {
        BftLeaderId(PublicKey::from_binary(&v[..]).expect("leader-id invalid format"))
    }
}

impl BftLeaderId {
    pub fn as_public_key(&self) -> &PublicKey<BftVerificationAlg> {
        &self.0
    }
}

impl Serialize for BftLeaderId {
    fn serialized_size(&self) -> usize {
        self.0.as_ref().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        serialize_public_key(&self.0, codec)
    }
}

impl DeserializeFromSlice for BftLeaderId {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        deserialize_public_key(codec).map(BftLeaderId)
    }
}

impl AsRef<[u8]> for BftLeaderId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
impl From<PublicKey<BftVerificationAlg>> for BftLeaderId {
    fn from(v: PublicKey<BftVerificationAlg>) -> Self {
        BftLeaderId(v)
    }
}

/// Praos Leader consisting of the KES public key and VRF public key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct GenesisPraosLeader {
    pub kes_public_key: PublicKey<SumEd25519_12>,
    pub vrf_public_key: PublicKey<RistrettoGroup2HashDh>,
}

impl GenesisPraosLeader {
    pub fn digest(&self) -> DigestOf<Blake2b256, Self> {
        DigestOf::digest_byteslice(
            &ByteBuilder::new()
                .bytes(self.vrf_public_key.as_ref())
                .bytes(self.kes_public_key.as_ref())
                .finalize()
                .as_byteslice(),
        )
    }
}

impl DeserializeFromSlice for GenesisPraosLeader {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let vrf_public_key = deserialize_public_key(codec)?;
        let kes_public_key = deserialize_public_key(codec)?;
        Ok(GenesisPraosLeader {
            kes_public_key,
            vrf_public_key,
        })
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    #[cfg(test)]
    use crate::testing::serialization::serialization_bijection;
    use chain_crypto::{testing, PublicKey, RistrettoGroup2HashDh, SecretKey, SumEd25519_12};
    use lazy_static::lazy_static;
    #[cfg(test)]
    use quickcheck::TestResult;
    use quickcheck::{quickcheck, Arbitrary, Gen};

    impl Arbitrary for Hash {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Hash(Arbitrary::arbitrary(g))
        }
    }
    impl Arbitrary for EitherEd25519SecretKey {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            if Arbitrary::arbitrary(g) {
                EitherEd25519SecretKey::Normal(Arbitrary::arbitrary(g))
            } else {
                EitherEd25519SecretKey::Extended(Arbitrary::arbitrary(g))
            }
        }
    }
    impl Arbitrary for GenesisPraosLeader {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            lazy_static! {
                static ref PK_KES: PublicKey<SumEd25519_12> =
                    testing::static_secret_key::<SumEd25519_12>().to_public();
            }

            let tcg = testing::TestCryptoGen::arbitrary(g);
            let mut rng = tcg.get_rng(0);
            let vrf_sk: SecretKey<RistrettoGroup2HashDh> = SecretKey::generate(&mut rng);
            GenesisPraosLeader {
                vrf_public_key: vrf_sk.to_public(),
                kes_public_key: PK_KES.clone(),
            }
        }
    }

    quickcheck! {
        fn leader_id_serialize_deserialize_biyection(leader_id: BftLeaderId) -> TestResult {
            serialization_bijection(leader_id)
        }
    }
}
