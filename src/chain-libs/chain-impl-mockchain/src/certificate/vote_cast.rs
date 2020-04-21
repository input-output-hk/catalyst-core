use crate::{
    certificate::{CertificateSlice, VotePlanId},
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
};
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VoteCast {
    vote_plan: VotePlanId,
    proposal_index: u8,
    payload: VoteCastPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VoteCastPayload {
    // TODO: add missing crypto here
    cryptographic_data: Vec<u8>,
}

impl VoteCast {
    pub fn new(vote_plan: VotePlanId, proposal_index: u8, payload: VoteCastPayload) -> Self {
        Self {
            vote_plan,
            proposal_index,
            payload,
        }
    }

    pub fn vote_plan(&self) -> &VotePlanId {
        &self.vote_plan
    }

    pub fn proposal_index(&self) -> u8 {
        self.proposal_index
    }

    pub fn payload(&self) -> &VoteCastPayload {
        &self.payload
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.bytes(self.vote_plan.as_ref()).u8(self.proposal_index)
        // .bytes(self.payload.as_ref())
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

impl VoteCastPayload {
    pub(crate) fn empty() -> Self {
        Self {
            cryptographic_data: Vec::new(),
        }
    }
}

impl AsRef<[u8]> for VoteCastPayload {
    fn as_ref(&self) -> &[u8] {
        self.cryptographic_data.as_slice()
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for VoteCast {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = false;
    type Auth = ();

    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            std::marker::PhantomData,
        )
    }

    fn payload_auth_data(_: &Self::Auth) -> PayloadAuthData<Self> {
        PayloadAuthData(Vec::with_capacity(0).into(), std::marker::PhantomData)
    }

    fn to_certificate_slice<'a>(p: PayloadSlice<'a, Self>) -> Option<CertificateSlice<'a>> {
        Some(CertificateSlice::from(p))
    }
}

/* Ser/De ******************************************************************* */

impl property::Serialize for VoteCast {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.serialize().as_slice())?;
        Ok(())
    }
}

impl Readable for VoteCast {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let vote_plan = <[u8; 32]>::read(buf)?.into();
        let proposal_index = buf.get_u8()?;
        let payload = VoteCastPayload {
            // TODO
            cryptographic_data: Vec::new(),
        };

        Ok(Self::new(vote_plan, proposal_index, payload))
    }
}
