use crate::vote::Choice;
use chain_core::packer::Codec;
use chain_core::property::ReadError;
use chain_vote::Ciphertext;
use std::hash::Hash;
use thiserror::Error;
use typed_bytes::{ByteArray, ByteBuilder};

/// the `PayloadType` to use for a vote plan
///
/// this defines how the vote must be published on chain.
/// Be careful because the default is set to `Public`.
///
/// ```
/// use chain_impl_mockchain::vote::PayloadType;
/// assert_eq!(PayloadType::Public, PayloadType::default());
/// ```
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum PayloadType {
    Public = 1,
    Private = 2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Payload {
    Public {
        choice: Choice,
    },
    Private {
        encrypted_vote: EncryptedVote,
        proof: ProofOfCorrectVote,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProofOfCorrectVote(chain_vote::ProofOfCorrectVote);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncryptedVote(chain_vote::EncryptedVote);

#[derive(Debug, Error)]
pub enum TryFromIntError {
    #[error("Found a `0` PayloadType. This is unexpected and known to be an error to read a 0.")]
    Zero,
    #[error("invalid value for a PayloadType")]
    InvalidValue { value: u8 },
}

impl Payload {
    pub fn public(choice: Choice) -> Self {
        Self::Public { choice }
    }

    pub fn private(encrypted_vote: EncryptedVote, proof: ProofOfCorrectVote) -> Self {
        Self::Private {
            encrypted_vote,
            proof,
        }
    }

    pub fn payload_type(&self) -> PayloadType {
        match self {
            Self::Public { .. } => PayloadType::Public,
            Self::Private { .. } => PayloadType::Private,
        }
    }

    pub(crate) fn serialize_in<T>(&self, bb: ByteBuilder<T>) -> ByteBuilder<T> {
        let payload_type = self.payload_type();

        let bb = bb.u8(payload_type as u8);

        match self {
            Self::Public { choice } => bb.u8(choice.as_byte()),
            Self::Private {
                encrypted_vote,
                proof,
            } => bb
                .sub(|bb| encrypted_vote.serialize_in(bb))
                .sub(|bb| proof.serialize_in(bb)),
        }
    }

    pub(crate) fn read(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let t = codec
            .get_u8()?
            .try_into()
            .map_err(|e: TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        match t {
            PayloadType::Public => codec.get_u8().map(Choice::new).map(Self::public),
            PayloadType::Private => {
                let encrypted_vote = EncryptedVote::read(codec)?;
                let proof = ProofOfCorrectVote::read(codec)?;
                Ok(Self::Private {
                    encrypted_vote,
                    proof,
                })
            }
        }
    }
}

impl ProofOfCorrectVote {
    pub(crate) fn from_inner(proof: chain_vote::ProofOfCorrectVote) -> Self {
        assert!(
            proof.len() <= u8::MAX as usize,
            "number of options is too large in an internally obtained proof"
        );
        Self(proof)
    }

    pub(super) fn as_inner(&self) -> &chain_vote::ProofOfCorrectVote {
        &self.0
    }

    pub(crate) fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        debug_assert!(self.0.len() <= u8::MAX as usize);
        bb.u8(self.0.len() as u8)
            .fold(self.0.ibas(), |bb, iba| bb.bytes(&iba.to_bytes()))
            .fold(self.0.ds(), |bb, d| bb.bytes(&d.to_bytes()))
            .fold(self.0.zwvs(), |bb, zwv| bb.bytes(&zwv.to_bytes()))
            .bytes(&self.0.r().to_bytes())
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }

    pub(crate) fn read(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        chain_vote::ProofOfCorrectVote::from_buffer(codec).map(Self)
    }
}

impl EncryptedVote {
    pub(crate) fn from_inner(vote: chain_vote::EncryptedVote) -> Self {
        Self(vote)
    }

    pub(super) fn as_inner(&self) -> &chain_vote::EncryptedVote {
        &self.0
    }

    pub(crate) fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.iter8(&self.0, |bb, ct| {
            let buffer = ct.to_bytes();
            bb.bytes(&buffer)
        })
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }

    pub(crate) fn read(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let len: usize = codec.get_u8()? as usize;
        let mut cypher_texts: Vec<Ciphertext> = Vec::new();
        for _ in 0..len {
            let ct_buf = codec.get_slice(Ciphertext::BYTES_LEN)?;
            cypher_texts.push(
                Ciphertext::from_bytes(ct_buf).ok_or_else(|| {
                    ReadError::StructureInvalid("Invalid private vote".to_string())
                })?,
            );
        }
        Ok(Self(cypher_texts))
    }
}

impl TryFrom<u8> for PayloadType {
    type Error = TryFromIntError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Err(TryFromIntError::Zero),
            1 => Ok(Self::Public),
            2 => Ok(Self::Private),
            _ => Err(TryFromIntError::InvalidValue { value }),
        }
    }
}

impl Default for PayloadType {
    fn default() -> Self {
        PayloadType::Public
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    use chain_vote::{Crs, ElectionPublicKey};
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for PayloadType {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            if g.next_u32() % 2 == 0 {
                Self::Public
            } else {
                Self::Private
            }
        }
    }

    impl Arbitrary for Payload {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            use chain_vote::{MemberCommunicationKey, MemberState, Vote};
            use rand_core::SeedableRng;

            match PayloadType::arbitrary(g) {
                PayloadType::Public => Payload::public(Choice::arbitrary(g)),
                PayloadType::Private => {
                    let mut seed = [0u8; 32];
                    g.fill_bytes(&mut seed);
                    let mut gen = rand_chacha::ChaCha20Rng::from_seed(seed);
                    let mc = MemberCommunicationKey::new(&mut gen);
                    let threshold = 1;
                    let h = Crs::from_hash(&seed);
                    let m = MemberState::new(&mut gen, threshold, &h, &[mc.to_public()], 0);
                    let participants = vec![m.public_key()];
                    let ek = ElectionPublicKey::from_participants(&participants);
                    let vote_options = 3;
                    let choice = g.next_u32() % vote_options;
                    let (vote, proof) = ek.encrypt_and_prove_vote(
                        &mut gen,
                        &h,
                        Vote::new(vote_options as usize, choice as usize),
                    );
                    Payload::private(
                        EncryptedVote::from_inner(vote),
                        ProofOfCorrectVote::from_inner(proof),
                    )
                }
            }
        }
    }
}
