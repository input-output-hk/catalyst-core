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
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct VoteTally {
    id: VotePlanId,
    payload: VoteTallyPayload,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum VoteTallyPayload {
    Public,
    Private { shares: TallyDecryptShares },
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TallyDecryptShares {
    inner: Box<[Box<[TallyDecryptShare]>]>,
}

impl VoteTallyPayload {
    pub fn payload_type(&self) -> PayloadType {
        match self {
            Self::Public => PayloadType::Public,
            Self::Private { .. } => PayloadType::Private,
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

    pub fn new_private(id: VotePlanId, shares: TallyDecryptShares) -> Self {
        Self {
            id,
            payload: VoteTallyPayload::Private { shares },
        }
    }

    pub fn id(&self) -> &VotePlanId {
        &self.id
    }

    pub fn tally_type(&self) -> PayloadType {
        self.payload.payload_type()
    }

    pub fn decrypt_shares(&self) -> Option<&TallyDecryptShares> {
        match &self.payload {
            VoteTallyPayload::Public => None,
            VoteTallyPayload::Private { shares } => Some(shares),
        }
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        use std::convert::TryInto;

        let bb = bb.bytes(self.id().as_ref()).u8(self.tally_type() as u8);

        match &self.payload {
            VoteTallyPayload::Public => bb,
            VoteTallyPayload::Private { shares } => {
                bb.u8(shares.inner.len().try_into().unwrap())
                    .fold(shares.inner.iter(), |bb, s| {
                        // Shares per proposal, n_members x n_options
                        let n_members = s.len().try_into().unwrap();
                        if n_members == 0 {
                            bb.u8(0).u8(0)
                        } else {
                            let n_options = s[0].options().try_into().unwrap();
                            bb.u8(n_members)
                                .u8(n_options)
                                .fold(s.iter(), |bb, s| bb.bytes(&s.to_bytes()))
                        }
                    })
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

impl TallyDecryptShares {
    pub fn new(shares: Vec<Vec<TallyDecryptShare>>) -> Self {
        Self {
            inner: shares
                .into_iter()
                .map(|s| s.into_boxed_slice())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub fn shares_for_proposal(&self, i: u8) -> Option<&[TallyDecryptShare]> {
        self.inner.get(i as usize).map(|s| s.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = &[TallyDecryptShare]> {
        self.inner.iter().map(|s| s.as_ref())
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

    fn to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
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
                    proposals.push(shares.into_boxed_slice());
                }
                let shares = TallyDecryptShares {
                    inner: proposals.into_boxed_slice(),
                };

                VoteTallyPayload::Private { shares }
            }
        };

        Ok(Self { id, payload })
    }
}
