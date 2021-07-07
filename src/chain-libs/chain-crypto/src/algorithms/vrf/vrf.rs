//! Verifiable Random Function (VRF) implementation
//! using the 2-Hash-DH verifiable oblivious PRF
//! defined in the Ouroboros Praos paper

use crate::ec::{GroupElement, Scalar};
use crate::hash::Blake2b256;
use rand_core::{CryptoRng, RngCore};
use std::hash::{Hash, Hasher};

use crate::key::PublicKeyError;
use crate::zkps::dleq;

/// VRF Secret Key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey {
    secret: Scalar,
    public: GroupElement,
    bytes: [u8; Scalar::BYTES_LEN],
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// VRF Public Key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(GroupElement, [u8; Self::BYTES_LEN]);

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state)
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// VRF Output (Point)
///
/// This is used to create an output generator tweaked by the VRF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputSeed(GroupElement);

/// VRF Proof of generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenOutputSeed {
    pub(crate) u: OutputSeed,
    dleq_proof: dleq::Zkp,
}

impl SecretKey {
    pub const BYTES_LEN: usize = Scalar::BYTES_LEN;
    /// Create a new random secret key
    pub fn random<T: RngCore + CryptoRng>(mut rng: T) -> Self {
        let sk = Scalar::random(&mut rng);
        let pk = GroupElement::generator() * &sk;
        SecretKey {
            secret: sk.clone(),
            public: pk,
            bytes: sk.to_bytes(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Serialize the secret key in binary form
    pub fn to_bytes(&self) -> [u8; Self::BYTES_LEN] {
        let mut v = [0u8; Self::BYTES_LEN];
        v.copy_from_slice(&self.secret.to_bytes());
        v
    }

    pub fn from_bytes(bytes: [u8; Self::BYTES_LEN]) -> Option<Self> {
        let sk = Scalar::from_bytes(&bytes)?;
        let pk = GroupElement::generator() * &sk;
        Some(SecretKey {
            secret: sk,
            public: pk,
            bytes,
        })
    }

    /// Get the verifiable output and the associated input base point.
    ///
    /// The following property hold between the return values:
    ///     Point * secret = OutputSeed
    pub fn verifiable_output(&self, input: &[u8]) -> (GroupElement, OutputSeed) {
        let m_point = GroupElement::from_hash(input);
        let u = &m_point * &self.secret;
        (m_point, OutputSeed(u))
    }

    /// Create a proof, for the given parameters; no check is made to make sure it's correct
    ///
    /// the proof is randomized, so need a secure RNG.
    ///
    /// use 'evaluate' for creating the proof directly from input
    fn prove<T: RngCore + CryptoRng>(
        &self,
        rng: &mut T,
        m_point: GroupElement,
        output: OutputSeed,
    ) -> ProvenOutputSeed {
        let dleq_proof = dleq::Zkp::generate(
            &GroupElement::generator(),
            &m_point,
            &self.public,
            &output.0,
            &self.secret,
            rng,
        );
        ProvenOutputSeed {
            u: output,
            dleq_proof,
        }
    }

    /// Generate a Proof
    ///
    pub fn evaluate<T: RngCore + CryptoRng>(&self, rng: &mut T, input: &[u8]) -> ProvenOutputSeed {
        let (m_point, output) = self.verifiable_output(input);
        self.prove(rng, m_point, output)
    }

    /// Get the public key associated with a secret key
    pub fn public(&self) -> PublicKey {
        PublicKey(self.public.clone(), self.public.to_bytes())
    }
}

impl PublicKey {
    pub const BYTES_LEN: usize = GroupElement::BYTES_LEN;
    pub fn from_bytes(input: &[u8]) -> Result<Self, PublicKeyError> {
        if input.len() != Self::BYTES_LEN {
            return Err(PublicKeyError::SizeInvalid);
        }
        let group_element = GroupElement::from_bytes(input);
        match group_element {
            None => Err(PublicKeyError::StructureInvalid),
            Some(pk) => Ok(PublicKey(pk.clone(), pk.to_bytes())),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.1
    }

    pub fn to_buffer(&self, output: &mut [u8]) {
        assert_eq!(output.len(), Self::BYTES_LEN);
        output.copy_from_slice(&self.0.to_bytes())
    }
}

impl ProvenOutputSeed {
    pub const BYTES_LEN: usize = dleq::Zkp::BYTES_LEN + GroupElement::BYTES_LEN;
    /// Verify a proof for a given public key and a data slice
    pub fn verify(&self, public_key: &PublicKey, input: &[u8]) -> bool {
        self.dleq_proof.verify(
            &GroupElement::generator(),
            &GroupElement::from_hash(&input),
            &public_key.0,
            &self.u.0,
        )
    }

    pub fn to_buffer(&self, output: &mut [u8]) {
        assert_eq!(output.len(), Self::BYTES_LEN);
        output[0..GroupElement::BYTES_LEN].copy_from_slice(&self.u.0.to_bytes());
        self.dleq_proof
            .write_to_bytes(&mut output[GroupElement::BYTES_LEN..]);
    }

    pub fn bytes(&self) -> [u8; Self::BYTES_LEN] {
        let mut output = [0u8; Self::BYTES_LEN];
        self.to_buffer(&mut output);
        output
    }

    pub fn from_bytes_unverified(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::BYTES_LEN {
            return None;
        }
        let u = GroupElement::from_bytes(&bytes[0..GroupElement::BYTES_LEN])?;
        let proof = dleq::Zkp::from_bytes(&bytes[GroupElement::BYTES_LEN..])?;
        Some(ProvenOutputSeed {
            u: OutputSeed(u),
            dleq_proof: proof,
        })
    }

    pub fn from_bytes(public_key: &PublicKey, bytes: &[u8], input: &[u8]) -> Option<Self> {
        let pos = Self::from_bytes_unverified(bytes)?;
        if pos.verify(public_key, input) {
            Some(pos)
        } else {
            None
        }
    }

    pub fn to_output(&self) -> OutputSeed {
        self.u.clone()
    }

    pub fn to_verifiable_output(&self, public_key: &PublicKey, input: &[u8]) -> Option<OutputSeed> {
        if self.verify(public_key, input) {
            Some(self.u.clone())
        } else {
            None
        }
    }
}

impl OutputSeed {
    /// Get the output for this input and a known suffix
    pub fn to_output(&self, input: &[u8], suffix: &[u8]) -> Blake2b256 {
        let mut buf = Vec::new();
        buf.extend_from_slice(input);
        buf.extend_from_slice(&self.0.to_bytes());
        buf.extend_from_slice(suffix);

        Blake2b256::new(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::{ProvenOutputSeed, PublicKey, SecretKey};
    use rand_core::{OsRng, RngCore};

    #[test]
    fn it_works() {
        let mut csprng: OsRng = OsRng;
        let sk = SecretKey::random(&mut csprng);
        let pk = sk.public();

        let sk_other = SecretKey::random(&mut csprng);
        let pk_other = sk_other.public();

        let mut b1 = [0u8; 10];
        for i in b1.iter_mut() {
            *i = csprng.next_u32() as u8;
        }
        let mut b2 = [0u8; 10];
        for i in b2.iter_mut() {
            *i = csprng.next_u32() as u8;
        }

        let proof = sk.evaluate(&mut csprng, &b1[..]);

        // make sure the test pass
        assert!(proof.verify(&pk, &b1[..]));

        // now try with false positive
        assert!(!proof.verify(&pk, &b2[..]));
        assert!(!proof.verify(&pk_other, &b1[..]));
        assert!(!proof.verify(&pk_other, &b2[..]));
    }

    #[test]
    fn serialisation() {
        let mut csprng: OsRng = OsRng;
        let sk = SecretKey::random(&mut csprng);
        let pk = sk.public();

        let serialised_sk = sk.to_bytes();
        let deserialised_sk = SecretKey::from_bytes(serialised_sk);

        assert!(deserialised_sk.is_some());
        assert_eq!(deserialised_sk.unwrap(), sk);

        let serialised_pk = pk.as_bytes();
        let deserialised_pk = PublicKey::from_bytes(serialised_pk);

        assert!(deserialised_pk.is_ok());
        assert_eq!(deserialised_pk.unwrap(), pk);

        let mut alpha = [0u8; 10];
        for i in alpha.iter_mut() {
            *i = csprng.next_u32() as u8;
        }

        let proof = sk.evaluate(&mut csprng, &alpha[..]);
        let serialised_proof = proof.bytes();
        let deserialised_proof = ProvenOutputSeed::from_bytes_unverified(&serialised_proof);
        let verified_deserialised_proof =
            ProvenOutputSeed::from_bytes(&pk, &serialised_proof, &alpha);

        assert!(deserialised_proof.is_some());
        assert!(verified_deserialised_proof.is_some());
        assert_eq!(deserialised_proof.unwrap(), proof);
    }

    #[test]
    fn to_buffer() {
        let mut csprng: OsRng = OsRng;
        let sk = SecretKey::random(&mut csprng);
        let pk = sk.public();

        let mut alpha = [0u8; 10];
        for i in alpha.iter_mut() {
            *i = csprng.next_u32() as u8;
        }

        let proof = sk.evaluate(&mut csprng, &alpha[..]);

        let mut buffer = [0u8; ProvenOutputSeed::BYTES_LEN + PublicKey::BYTES_LEN];
        pk.to_buffer(&mut buffer[..PublicKey::BYTES_LEN]);
        proof.to_buffer(&mut buffer[PublicKey::BYTES_LEN..]);

        let deserialised_pk = PublicKey::from_bytes(&buffer[..PublicKey::BYTES_LEN]);

        assert!(deserialised_pk.is_ok());
        assert_eq!(deserialised_pk.unwrap(), pk);

        let deserialised_proof =
            ProvenOutputSeed::from_bytes_unverified(&buffer[PublicKey::BYTES_LEN..]);

        assert!(deserialised_proof.is_some());
        assert!(deserialised_proof.unwrap().verify(&pk, &alpha));
    }
}
