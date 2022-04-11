use crate::{
    certificate::{CertificateSlice, VotePlanId},
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
    vote,
};
use chain_core::{
    packer::Codec,
    property::{Deserialize, DeserializeFromSlice, ReadError, Serialize, WriteError},
};
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VoteCast {
    vote_plan: VotePlanId,
    proposal_index: u8,
    payload: vote::Payload,
}

impl VoteCast {
    pub fn new(vote_plan: VotePlanId, proposal_index: u8, payload: vote::Payload) -> Self {
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

    pub fn payload(&self) -> &vote::Payload {
        &self.payload
    }

    pub(crate) fn into_payload(self) -> vote::Payload {
        self.payload
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        let bb = bb.bytes(self.vote_plan.as_ref()).u8(self.proposal_index);
        self.payload.serialize_in(bb)
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
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

    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

/* Ser/De ******************************************************************* */

impl Serialize for VoteCast {
    fn serialized_size(&self) -> usize {
        self.serialize().as_slice().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.serialize().as_slice())
    }
}

impl DeserializeFromSlice for VoteCast {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let vote_plan = <[u8; 32]>::deserialize(codec)?.into();
        let proposal_index = codec.get_u8()?;
        let payload = vote::Payload::read(codec)?;

        Ok(Self::new(vote_plan, proposal_index, payload))
    }
}
