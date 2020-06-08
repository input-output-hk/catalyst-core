//! Module provides cryptographic utilities and types related to
//! the user keys.
//!
use chain_core::mempack::{read_mut_slice, ReadBuf, ReadError, Readable};
use chain_core::property;
use chain_crypto as crypto;
use chain_crypto::{
    digest::DigestOf, AsymmetricKey, AsymmetricPublicKey, Blake2b256, Curve25519_2HashDH, Ed25519,
    PublicKey, SecretKey, SigningAlgorithm, SumEd25519_12, VerificationAlgorithm,
};
use rand_core::{CryptoRng, RngCore};
use typed_bytes::ByteBuilder;

use chain_core::packer::Codec;
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
    mut writer: W,
) -> Result<(), std::io::Error> {
    writer.write_all(key.as_ref())
}
#[inline]
pub fn serialize_signature<A: VerificationAlgorithm, T, W: std::io::Write>(
    signature: &crypto::Signature<T, A>,
    mut writer: W,
) -> Result<(), std::io::Error> {
    writer.write_all(signature.as_ref())
}
#[inline]
pub fn deserialize_public_key<'a, A>(
    buf: &mut ReadBuf<'a>,
) -> Result<crypto::PublicKey<A>, ReadError>
where
    A: AsymmetricPublicKey,
{
    let mut bytes = vec![0u8; A::PUBLIC_KEY_SIZE];
    read_mut_slice(buf, &mut bytes[..])?;
    crypto::PublicKey::from_binary(&bytes).map_err(chain_crypto_pub_err)
}
#[inline]
pub fn deserialize_signature<'a, A, T>(
    buf: &mut ReadBuf<'a>,
) -> Result<crypto::Signature<T, A>, ReadError>
where
    A: VerificationAlgorithm,
{
    let mut bytes = vec![0u8; A::SIGNATURE_SIZE];
    read_mut_slice(buf, &mut bytes[..])?;
    crypto::Signature::from_binary(&bytes).map_err(chain_crypto_sig_err)
}

pub fn make_signature<T, A>(
    spending_key: &crypto::SecretKey<A>,
    data: &T,
) -> crypto::Signature<T, A::PubAlg>
where
    A: SigningAlgorithm,
    <A as AsymmetricKey>::PubAlg: VerificationAlgorithm,
    T: property::Serialize,
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
    T: property::Serialize,
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
    T: property::Serialize,
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

pub fn signed_new<T: property::Serialize, A: SigningAlgorithm>(
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

impl<T: property::Serialize, A: VerificationAlgorithm> property::Serialize for Signed<T, A>
where
    std::io::Error: From<T::Error>,
{
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        self.data.serialize(&mut writer)?;
        serialize_signature(&self.sig, &mut writer)?;
        Ok(())
    }
}

impl<T: Readable, A: VerificationAlgorithm> Readable for Signed<T, A> {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        Ok(Signed {
            data: T::read(buf)?,
            sig: deserialize_signature(buf)?,
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

impl property::Serialize for Hash {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.0.as_hash_bytes())?;
        Ok(())
    }
}

impl property::Deserialize for Hash {
    type Error = std::io::Error;
    fn deserialize<R: std::io::BufRead>(mut reader: R) -> Result<Self, Self::Error> {
        let mut buffer = [0; crypto::Blake2b256::HASH_SIZE];
        reader.read_exact(&mut buffer)?;
        Ok(Hash(crypto::Blake2b256::from(buffer)))
    }
}

impl Readable for Hash {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let bytes = <[u8; crypto::Blake2b256::HASH_SIZE]>::read(buf)?;
        Ok(Hash(crypto::Blake2b256::from(bytes)))
    }
}

impl property::BlockId for Hash {
    fn zero() -> Hash {
        Hash(crypto::Blake2b256::from([0; crypto::Blake2b256::HASH_SIZE]))
    }
}

impl property::FragmentId for Hash {}

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

impl property::Serialize for BftLeaderId {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        serialize_public_key(&self.0, writer)
    }
}

impl property::Deserialize for BftLeaderId {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        let mut codec = Codec::new(reader);
        let size: usize = 32;
        let bytes = codec.get_bytes(size)?;
        let mut buff = ReadBuf::from(&bytes);
        match deserialize_public_key(&mut buff) {
            Ok(pk) => Ok(BftLeaderId(pk)),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error reading LeaderId public key: {}", e),
            )),
        }
    }
}

impl Readable for BftLeaderId {
    fn read<'a>(reader: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        deserialize_public_key(reader).map(BftLeaderId)
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
pub struct GenesisPraosLeader {
    pub kes_public_key: PublicKey<SumEd25519_12>,
    pub vrf_public_key: PublicKey<Curve25519_2HashDH>,
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

impl Readable for GenesisPraosLeader {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let vrf_public_key = deserialize_public_key(buf)?;
        let kes_public_key = deserialize_public_key(buf)?;
        Ok(GenesisPraosLeader {
            vrf_public_key,
            kes_public_key,
        })
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    use chain_crypto::{testing, Curve25519_2HashDH, PublicKey, SecretKey, SumEd25519_12};
    #[cfg(test)]
    use chain_test_utils::property::serialization_bijection;
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
            let vrf_sk: SecretKey<Curve25519_2HashDH> = SecretKey::generate(&mut rng);
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
