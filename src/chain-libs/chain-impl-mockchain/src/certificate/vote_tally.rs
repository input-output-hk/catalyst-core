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
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct VoteTally {
    id: VotePlanId,
    payload: VoteTallyPayload,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum VoteTallyPayload {
    Public,
}

#[derive(Debug, Clone)]
pub enum TallyProof {
    Public {
        id: CommitteeId,
        signature: SingleAccountBindingSignature,
    },
}

impl VoteTallyPayload {
    pub fn payload_type(&self) -> PayloadType {
        match self {
            Self::Public => PayloadType::Public,
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

    pub fn id(&self) -> &VotePlanId {
        &self.id
    }

    pub fn tally_type(&self) -> PayloadType {
        self.payload.payload_type()
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.bytes(self.id().as_ref())
            .u8(self.payload.payload_type() as u8)
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

impl TallyProof {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        match self {
            Self::Public { id, signature } => bb.u8(0).bytes(id.as_ref()).bytes(signature.as_ref()),
        }
    }

    pub fn verify<'a>(
        &self,
        tally: &VoteTally,
        verify_data: &TransactionBindingAuthData<'a>,
    ) -> Verification {
        match self {
            Self::Public { id, signature } => {
                if tally.tally_type() != PayloadType::Public {
                    Verification::Failed
                } else {
                    let pk = id.public_key();

                    signature.verify_slice(&pk, verify_data)
                }
            }
        }
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for VoteTally {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true; // TODO: true it is the Committee signatures
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
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        match buf.peek_u8()? {
            0 => {
                let _ = buf.get_u8()?;
                let id = CommitteeId::read(buf)?;
                let signature = SingleAccountBindingSignature::read(buf)?;
                Ok(Self::Public { id, signature })
            }
            _ => Err(ReadError::StructureInvalid(
                "Unknown Tally proof type".to_owned(),
            )),
        }
    }
}

impl Readable for VoteTally {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        use std::convert::TryInto as _;

        let id = <[u8; 32]>::read(buf)?.into();
        let payload_type = buf
            .get_u8()?
            .try_into()
            .map_err(|e: TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        let payload = match payload_type {
            PayloadType::Public => VoteTallyPayload::Public,
        };

        Ok(Self { id, payload })
    }
}
