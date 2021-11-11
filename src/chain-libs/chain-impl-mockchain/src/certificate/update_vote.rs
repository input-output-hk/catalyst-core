use crate::{
    certificate::{BftLeaderBindingSignature, CertificateSlice},
    key::BftLeaderId,
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
};

use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use typed_bytes::{ByteArray, ByteBuilder};

pub type UpdateVoterId = BftLeaderId;
pub type UpdateProposalId = crate::fragment::FragmentId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpdateVote {
    proposal_id: UpdateProposalId,
    voter_id: UpdateVoterId,
}

impl UpdateVote {
    pub fn new(proposal_id: UpdateProposalId, voter_id: UpdateVoterId) -> Self {
        Self {
            proposal_id,
            voter_id,
        }
    }

    pub fn proposal_id(&self) -> &UpdateProposalId {
        &self.proposal_id
    }

    pub fn voter_id(&self) -> &UpdateVoterId {
        &self.voter_id
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.bytes(self.proposal_id.as_ref())
            .bytes(self.voter_id.as_ref())
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for UpdateVote {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true;
    type Auth = BftLeaderBindingSignature;

    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            std::marker::PhantomData,
        )
    }

    fn payload_auth_data(auth: &Self::Auth) -> PayloadAuthData<Self> {
        let bb = ByteBuilder::<Self>::new()
            .bytes(auth.as_ref())
            .finalize_as_vec();
        PayloadAuthData(bb.into(), std::marker::PhantomData)
    }

    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

/* Ser/De ******************************************************************* */

impl property::Serialize for UpdateVote {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::new(writer);
        self.proposal_id.serialize(&mut codec)?;
        self.voter_id.serialize(&mut codec)?;
        Ok(())
    }
}

impl Readable for UpdateVote {
    fn read(buf: &mut ReadBuf) -> Result<Self, ReadError> {
        let proposal_id = UpdateProposalId::read(buf)?;
        let voter_id = UpdateVoterId::read(buf)?;

        Ok(Self::new(proposal_id, voter_id))
    }
}
