use crate::vote::Choice;
use chain_core::mempack::{ReadBuf, ReadError};
use chain_vote::shvzk;
use chain_vote::{Ciphertext, Scalar};
use std::convert::{TryFrom, TryInto as _};
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

pub use chain_vote::EncryptedVote;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProofOfCorrectVote(chain_vote::ProofOfCorrectVote);

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
                .iter8(encrypted_vote, |bb, ct| {
                    let buffer = ct.to_bytes();
                    bb.bytes(&buffer)
                })
                .sub(|bb| proof.serialize_in(bb)),
        }
    }

    pub(crate) fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let t = buf
            .get_u8()?
            .try_into()
            .map_err(|e: TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        match t {
            PayloadType::Public => buf.get_u8().map(Choice::new).map(Self::public),
            PayloadType::Private => {
                let len: usize = buf.get_u8()? as usize;
                let mut cypher_texts: Vec<Ciphertext> = Vec::new();
                for _ in 0..len {
                    let ct_buf = buf.get_slice(Ciphertext::BYTES_LEN)?;
                    cypher_texts.push(Ciphertext::from_bytes(ct_buf).ok_or_else(|| {
                        ReadError::StructureInvalid("Invalid private vote".to_string())
                    })?);
                }
                let proof = ProofOfCorrectVote::read(buf)?;
                Ok(Self::Private {
                    encrypted_vote: cypher_texts,
                    proof,
                })
            }
        }
    }
}

impl ProofOfCorrectVote {
    #[cfg(any(test, feature = "property-test-api"))]
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

    pub(crate) fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let bits = buf.get_u8()? as usize;
        let mut ibas = Vec::with_capacity(bits);
        for _ in 0..bits {
            let elem_buf = buf.get_slice(shvzk::IBA::BYTES_LEN)?;
            let iba = shvzk::IBA::from_bytes(elem_buf)
                .ok_or_else(|| ReadError::StructureInvalid("Invalid IBA component".to_string()))?;
            ibas.push(iba);
        }
        let mut bs = Vec::with_capacity(bits);
        for _ in 0..bits {
            let elem_buf = buf.get_slice(Ciphertext::BYTES_LEN)?;
            let ciphertext = Ciphertext::from_bytes(elem_buf).ok_or_else(|| {
                ReadError::StructureInvalid("Invalid encoded ciphertext".to_string())
            })?;
            bs.push(ciphertext);
        }
        let mut zwvs = Vec::with_capacity(bits);
        for _ in 0..bits {
            let elem_buf = buf.get_slice(shvzk::ZWV::BYTES_LEN)?;
            let zwv = shvzk::ZWV::from_bytes(elem_buf)
                .ok_or_else(|| ReadError::StructureInvalid("Invalid ZWV component".to_string()))?;
            zwvs.push(zwv);
        }
        let r_buf = buf.get_slice(Scalar::BYTES_LEN)?;
        let r = Scalar::from_bytes(r_buf).ok_or_else(|| {
            ReadError::StructureInvalid("Invalid Proof encoded R scalar".to_string())
        })?;
        Ok(Self(chain_vote::ProofOfCorrectVote::from_parts(
            ibas, bs, zwvs, r,
        )))
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
            use chain_vote::{
                encrypt_vote, EncryptingVoteKey, MemberCommunicationKey, MemberState, Vote, CRS,
            };
            use rand_core::SeedableRng;

            match PayloadType::arbitrary(g) {
                PayloadType::Public => Payload::public(Choice::arbitrary(g)),
                PayloadType::Private => {
                    let mut seed = [0u8; 32];
                    g.fill_bytes(&mut seed);
                    let mut gen = rand_chacha::ChaCha20Rng::from_seed(seed);
                    let mc = MemberCommunicationKey::new(&mut gen);
                    let threshold = 1;
                    let h = CRS::random(&mut gen);
                    let m = MemberState::new(&mut gen, threshold, &h, &[mc.to_public()], 0);
                    let participants = vec![m.public_key()];
                    let ek = EncryptingVoteKey::from_participants(&participants);
                    let vote_options = 3;
                    let choice = g.next_u32() % vote_options;
                    let (vote, proof) = encrypt_vote(
                        &mut gen,
                        &ek,
                        Vote::new(vote_options as usize, choice as usize),
                    );
                    Payload::private(vote, ProofOfCorrectVote::from_inner(proof))
                }
            }
        }
    }
}
