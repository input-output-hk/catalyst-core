use crate::{
    certificate::{CertificateSlice, VotePlanId},
    transaction::{
        Payload, PayloadAuthData, PayloadData, PayloadSlice, SingleAccountBindingSignature,
        TransactionBindingAuthData,
    },
    vote::{CommitteeId, PayloadType, TryFromIntError},
};
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use chain_crypto::Verification;
use chain_vote::TallyDecryptShare;
use thiserror::Error;
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct VoteTally {
    id: VotePlanId,
    payload: VoteTallyPayload,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum VoteTallyPayload {
    Public,
    Private { inner: DecryptedPrivateTally },
}

#[derive(Debug, Clone)]
pub enum TallyProof {
    Public {
        id: CommitteeId,
        signature: SingleAccountBindingSignature,
    },

    Private {
        id: CommitteeId,
        signature: SingleAccountBindingSignature,
    },
}

#[derive(Debug, Error)]
#[error("decrypt_shares in the proposal should have the same options amount")]
pub struct DecryptedPrivateTallyError {}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DecryptedPrivateTally {
    inner: Box<[DecryptedPrivateTallyProposal]>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DecryptedPrivateTallyProposal {
    pub decrypt_shares: Box<[TallyDecryptShare]>,
    pub tally_result: Box<[u64]>,
}

impl VoteTallyPayload {
    pub fn payload_type(&self) -> PayloadType {
        match self {
            Self::Public => PayloadType::Public,
            Self::Private { .. } => PayloadType::Private,
        }
    }

    pub fn payload_decrypted(&self) -> Option<&DecryptedPrivateTally> {
        match self {
            Self::Public => None,
            Self::Private { inner } => Some(inner),
        }
    }
}

impl VoteTally {
    pub fn new_public(id: VotePlanId) -> Self {
        Self {
            id,
            payload: VoteTallyPayload::Public,
        }
    }

    pub fn new_private(id: VotePlanId, decrypted_tally: DecryptedPrivateTally) -> Self {
        Self {
            id,
            payload: VoteTallyPayload::Private {
                inner: decrypted_tally,
            },
        }
    }

    pub fn id(&self) -> &VotePlanId {
        &self.id
    }

    pub fn tally_type(&self) -> PayloadType {
        self.payload.payload_type()
    }

    pub fn tally_decrypted(&self) -> Option<&DecryptedPrivateTally> {
        self.payload.payload_decrypted()
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        use std::convert::TryInto;

        let bb = bb.bytes(self.id().as_ref()).u8(self.tally_type() as u8);

        match &self.payload {
            VoteTallyPayload::Public => bb,
            VoteTallyPayload::Private { inner: proposals } => {
                bb.u8(proposals.inner.len().try_into().unwrap()).fold(
                    proposals.inner.iter(),
                    |bb, proposal| {
                        // Shares per proposal, n_members x n_options
                        let n_members = proposal.decrypt_shares.len().try_into().unwrap();
                        if n_members == 0 {
                            bb.u8(0).u8(0)
                        } else {
                            let n_options = proposal.tally_result.len().try_into().unwrap();
                            bb.u8(n_members)
                                .u8(n_options)
                                .fold(proposal.decrypt_shares.iter(), |bb, s| {
                                    bb.bytes(&s.to_bytes())
                                })
                                .fold(proposal.tally_result.iter(), |bb, count| bb.u64(*count))
                        }
                    },
                )
            }
        }
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

impl TallyProof {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        match self {
            Self::Public { id, signature } => bb.u8(0).bytes(id.as_ref()).bytes(signature.as_ref()),
            Self::Private { id, signature } => {
                bb.u8(1).bytes(id.as_ref()).bytes(signature.as_ref())
            }
        }
    }

    pub fn verify<'a>(
        &self,
        tally_type: PayloadType,
        verify_data: &TransactionBindingAuthData<'a>,
    ) -> Verification {
        match self {
            Self::Public { id, signature } => {
                if tally_type != PayloadType::Public {
                    Verification::Failed
                } else {
                    let pk = id.public_key();
                    signature.verify_slice(&pk, verify_data)
                }
            }
            Self::Private { id, signature } => {
                if tally_type != PayloadType::Private {
                    Verification::Failed
                } else {
                    let pk = id.public_key();
                    signature.verify_slice(&pk, verify_data)
                }
            }
        }
    }
}

impl DecryptedPrivateTally {
    pub fn new(
        proposals: Vec<DecryptedPrivateTallyProposal>,
    ) -> Result<Self, DecryptedPrivateTallyError> {
        let is_valid = proposals
            .iter()
            .all(|proposal| match proposal.decrypt_shares.first() {
                Some(share) => proposal
                    .decrypt_shares
                    .iter()
                    .all(|el| el.options() == share.options()),
                None => true,
            });

        is_valid
            .then(|| Self {
                inner: proposals.into_boxed_slice(),
            })
            .ok_or(DecryptedPrivateTallyError {})
    }

    pub fn iter(&self) -> impl Iterator<Item = &DecryptedPrivateTallyProposal> {
        self.inner.iter()
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for VoteTally {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true;
    type Auth = TallyProof;

    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            std::marker::PhantomData,
        )
    }

    fn payload_auth_data(auth: &Self::Auth) -> PayloadAuthData<Self> {
        PayloadAuthData(
            auth.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            std::marker::PhantomData,
        )
    }

    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

/* Ser/De ******************************************************************* */

impl property::Serialize for VoteTally {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.serialize().as_slice())?;
        Ok(())
    }
}

impl Readable for TallyProof {
    fn read(buf: &mut ReadBuf) -> Result<Self, ReadError> {
        match buf.get_u8()? {
            0 => {
                let id = CommitteeId::read(buf)?;
                let signature = SingleAccountBindingSignature::read(buf)?;
                Ok(Self::Public { id, signature })
            }
            1 => {
                let id = CommitteeId::read(buf)?;
                let signature = SingleAccountBindingSignature::read(buf)?;
                Ok(Self::Private { id, signature })
            }
            _ => Err(ReadError::StructureInvalid(
                "Unknown Tally proof type".to_owned(),
            )),
        }
    }
}

impl Readable for VoteTally {
    fn read(buf: &mut ReadBuf) -> Result<Self, ReadError> {
        use std::convert::TryInto as _;

        let id = <[u8; 32]>::read(buf)?.into();
        let payload_type = buf
            .get_u8()?
            .try_into()
            .map_err(|e: TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        let payload = match payload_type {
            PayloadType::Public => VoteTallyPayload::Public,
            PayloadType::Private => {
                let proposals_number = buf.get_u8()? as usize;
                let mut proposals = Vec::with_capacity(proposals_number);
                for _i in 0..proposals_number {
                    let shares_number = buf.get_u8()? as usize;
                    let options_number = buf.get_u8()? as usize;
                    let share_bytes = TallyDecryptShare::bytes_len(options_number);
                    let mut shares = Vec::with_capacity(shares_number);
                    for _j in 0..shares_number {
                        let s_buf = buf.get_slice(share_bytes)?;
                        let share = TallyDecryptShare::from_bytes(s_buf).ok_or_else(|| {
                            ReadError::StructureInvalid(
                                "invalid decrypt share structure".to_owned(),
                            )
                        })?;
                        shares.push(share);
                    }
                    let mut decrypted = Vec::with_capacity(options_number);
                    for _j in 0..options_number {
                        decrypted.push(buf.get_u64()?);
                    }
                    let shares = shares.into_boxed_slice();
                    let decrypted = decrypted.into_boxed_slice();
                    proposals.push(DecryptedPrivateTallyProposal {
                        decrypt_shares: shares,
                        tally_result: decrypted,
                    });
                }

                VoteTallyPayload::Private {
                    inner: DecryptedPrivateTally::new(proposals)
                        .map_err(|err| ReadError::InvalidData(err.to_string()))?,
                }
            }
        };

        Ok(Self { id, payload })
    }
}
