use crate::certificate::{TallyProof, VoteTallyPayload};
use crate::{
    certificate::{CertificateSlice, VotePlanId},
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
    vote::{PayloadType, TryFromIntError},
};
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PrivateVoteTally {
    pub id: VotePlanId,
    pub payload: VoteTallyPayload,
    pub shares: Vec<Vec<chain_vote::TallyDecryptShare>>,
}

impl PrivateVoteTally {
    pub fn new_private(id: VotePlanId, shares: Vec<Vec<chain_vote::TallyDecryptShare>>) -> Self {
        Self {
            id,
            payload: VoteTallyPayload::Private,
            shares,
        }
    }

    pub fn id(&self) -> &VotePlanId {
        &self.id
    }

    pub fn tally_type(&self) -> PayloadType {
        self.payload.payload_type()
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        let mut bb = bb
            .bytes(self.id().as_ref())
            .u8(self.payload.payload_type() as u8)
            .u64(self.shares.len() as u64);
        for shares in &self.shares {
            bb = bb.u64(shares.len() as u64).bytes(
                &shares
                    .iter()
                    .flat_map(|s| s.serialize().as_slice().to_vec())
                    .collect::<Vec<u8>>(),
            );
        }
        bb
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for PrivateVoteTally {
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

impl property::Serialize for PrivateVoteTally {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.serialize().as_slice())?;
        Ok(())
    }
}

impl Readable for PrivateVoteTally {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        use std::convert::TryInto as _;

        let id = <[u8; 32]>::read(buf)?.into();
        let payload_type = buf
            .get_u8()?
            .try_into()
            .map_err(|e: TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        let payload = match payload_type {
            PayloadType::Public => VoteTallyPayload::Public,
            PayloadType::Private => VoteTallyPayload::Private,
        };

        let all_shares_len = buf.get_u64()?;
        let mut all_shares = Vec::with_capacity(all_shares_len as usize);
        for _ in 0..all_shares_len {
            let proposal_shares_len = buf.get_u64()?;
            let mut proposal_shares = Vec::with_capacity(proposal_shares_len as usize);
            for _ in 0..proposal_shares_len {
                proposal_shares.push(chain_vote::TallyDecryptShare::read(buf)?);
            }
            all_shares.push(proposal_shares);
        }

        Ok(Self {
            id,
            payload,
            shares: all_shares,
        })
    }
}
