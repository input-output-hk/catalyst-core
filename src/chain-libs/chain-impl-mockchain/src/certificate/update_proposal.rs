use crate::fragment::config::ConfigParams;
use crate::transaction::SingleAccountBindingSignature;

use crate::{
    certificate::CertificateSlice,
    key::BftLeaderId,
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
};
use chain_core::property::WriteError;
use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, ReadError, Serialize},
};
use typed_bytes::{ByteArray, ByteBuilder};

pub type UpdateProposerId = BftLeaderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProposal {
    changes: ConfigParams,
    proposer_id: UpdateProposerId,
}

impl UpdateProposal {
    pub fn new(changes: ConfigParams, proposer_id: UpdateProposerId) -> Self {
        Self {
            changes,
            proposer_id,
        }
    }

    pub fn changes(&self) -> &ConfigParams {
        &self.changes
    }

    pub fn proposer_id(&self) -> &UpdateProposerId {
        &self.proposer_id
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        // Should be impossible to fail serialization
        bb.bytes(self.changes.serialize_as_vec().as_ref().unwrap())
            .bytes(self.proposer_id.as_ref())
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

pub type BftLeaderBindingSignature = SingleAccountBindingSignature;

/* Auth/Payload ************************************************************* */

impl Payload for UpdateProposal {
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

impl Serialize for UpdateProposal {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        self.changes.serialize(codec)?;
        self.proposer_id.serialize(codec)
    }
}

impl DeserializeFromSlice for UpdateProposal {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let changes = ConfigParams::deserialize_from_slice(codec)?;
        let proposer_id = UpdateProposerId::deserialize_from_slice(codec)?;

        Ok(Self::new(changes, proposer_id))
    }
}
