use super::components::VrfProof;
use super::cstruct;
use super::deconstruct::{BftProof, Common, GenesisPraosProof, Proof};
use super::version::BlockVersion;

use crate::certificate::PoolId;
use crate::chaineval::{ConsensusEvalContext, HeaderContentEvalContext};
use crate::chaintypes::{ChainLength, HeaderId};
use crate::date::BlockDate;
use crate::fragment::{BlockContentHash, BlockContentSize};
use crate::key::BftLeaderId;
use crate::leadership;

use std::fmt::{self, Debug};
use std::num::NonZeroUsize;

use chain_core::property::WriteError;
pub use cstruct::HeaderError;

/// Finalized Unsigned Header
#[derive(Clone, PartialEq, Eq)]
pub struct HeaderUnsigned(pub(super) cstruct::Header);

/// Finalized Genesis-Praos Header
#[derive(Clone, PartialEq, Eq)]
pub struct HeaderGenesisPraos(pub(super) cstruct::Header);

/// Finalized BFT Header
#[derive(Clone, PartialEq, Eq)]
pub struct HeaderBft(pub(super) cstruct::Header);

#[derive(Clone, PartialEq, Eq)]
pub enum Header {
    Unsigned(HeaderUnsigned),
    GenesisPraos(HeaderGenesisPraos),
    Bft(HeaderBft),
}

impl HeaderUnsigned {
    pub fn id(&self) -> HeaderId {
        HeaderId::hash_bytes(self.0.as_slice().as_slice())
    }

    pub fn generalize(self) -> Header {
        Header::Unsigned(self)
    }
}

impl HeaderBft {
    pub fn id(&self) -> HeaderId {
        HeaderId::hash_bytes(self.0.as_slice().as_slice())
    }

    pub fn generalize(self) -> Header {
        Header::Bft(self)
    }
}

impl HeaderGenesisPraos {
    pub fn id(&self) -> HeaderId {
        HeaderId::hash_bytes(self.0.as_slice().as_slice())
    }

    pub fn generalize(self) -> Header {
        Header::GenesisPraos(self)
    }
}

/// Header description
#[derive(Clone)]
pub struct HeaderDesc {
    pub id: HeaderId,
    pub date: BlockDate,
    pub height: ChainLength,
}

impl fmt::Debug for HeaderDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{:08x}-{}.{}",
            self.id, self.height.0, self.date.epoch, self.date.slot_id,
        )
    }
}
impl fmt::Display for HeaderDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.8}-{:08x}-{}.{}",
            self.id.to_string(),
            self.height.0,
            self.date.epoch,
            self.date.slot_id,
        )
    }
}

impl Header {
    pub fn id(&self) -> HeaderId {
        match self {
            Header::Unsigned(h) => h.id(),
            Header::GenesisPraos(h) => h.id(),
            Header::Bft(h) => h.id(),
        }
    }

    pub fn size(&self) -> NonZeroUsize {
        self.block_version().get_size()
    }

    // deprecated: use .id()
    #[inline]
    pub fn hash(&self) -> HeaderId {
        self.id()
    }

    pub fn description(&self) -> HeaderDesc {
        HeaderDesc {
            id: self.id(),
            date: self.block_date(),
            height: self.chain_length(),
        }
    }

    fn get_cstruct(&self) -> cstruct::HeaderSlice<'_> {
        match self {
            Header::Unsigned(h) => h.0.as_slice(),
            Header::GenesisPraos(h) => h.0.as_slice(),
            Header::Bft(h) => h.0.as_slice(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.get_cstruct().as_slice()
    }

    pub fn as_auth_slice(&self) -> &[u8] {
        match self {
            Header::Unsigned(_) => self.get_cstruct().as_slice(),
            Header::Bft(_) => self.get_cstruct().slice_bft_auth(),
            Header::GenesisPraos(_) => self.get_cstruct().slice_gp_auth(),
        }
    }

    #[inline]
    pub fn block_version(&self) -> BlockVersion {
        match self {
            Header::Unsigned(_) => BlockVersion::Genesis,
            Header::GenesisPraos(_) => BlockVersion::KesVrfproof,
            Header::Bft(_) => BlockVersion::Ed25519Signed,
        }
    }

    #[inline]
    pub fn block_date(&self) -> BlockDate {
        let cs = self.get_cstruct();
        let epoch = cs.date_epoch();
        let slot_id = cs.date_slotid();
        BlockDate { epoch, slot_id }
    }

    #[inline]
    pub fn block_content_hash(&self) -> BlockContentHash {
        self.get_cstruct().content_hash().into()
    }

    #[inline]
    pub fn block_content_size(&self) -> BlockContentSize {
        self.get_cstruct().content_size()
    }

    #[inline]
    pub fn block_parent_hash(&self) -> HeaderId {
        self.get_cstruct().parent_hash().into()
    }

    #[inline]
    pub fn chain_length(&self) -> ChainLength {
        self.get_cstruct().height().into()
    }

    pub fn from_slice(slice: &[u8]) -> Result<Header, HeaderError> {
        let hdr_slice = cstruct::HeaderSlice::from_slice(slice)?;
        let hdr = hdr_slice.to_owned();
        match BlockVersion::from_u16(hdr.version()).expect("header slice only know version") {
            BlockVersion::Genesis => Ok(Header::Unsigned(HeaderUnsigned(hdr))),
            BlockVersion::Ed25519Signed => Ok(Header::Bft(HeaderBft(hdr))),
            BlockVersion::KesVrfproof => Ok(Header::GenesisPraos(HeaderGenesisPraos(hdr))),
        }
    }

    pub fn to_raw(&self) -> Box<[u8]> {
        let mut v = Vec::with_capacity(self.size().get());
        v.extend_from_slice(self.as_slice());
        v.into()
    }

    pub fn common(&self) -> Common {
        Common {
            block_version: self.block_version(),
            block_date: self.block_date(),
            block_content_size: self.block_content_size(),
            block_content_hash: self.block_content_hash(),
            block_parent_hash: self.block_parent_hash(),
            chain_length: self.chain_length(),
        }
    }

    pub fn proof(&self) -> Proof {
        match self.block_version() {
            BlockVersion::Genesis => Proof::None,
            BlockVersion::Ed25519Signed => Proof::Bft(BftProof {
                leader_id: self.get_cstruct().bft_leader_id().into(),
                signature: self.get_cstruct().bft_signature().into(),
            }),
            BlockVersion::KesVrfproof => Proof::GenesisPraos(GenesisPraosProof {
                node_id: self.get_cstruct().gp_node_id().into(),
                vrf_proof: VrfProof(self.get_cstruct().gp_vrf_proof()),
                kes_proof: self.get_cstruct().gp_kes_signature().into(),
            }),
        }
    }

    #[inline]
    pub fn get_stakepool_id(&self) -> Option<PoolId> {
        match self.block_version() {
            BlockVersion::KesVrfproof => Some(self.get_cstruct().gp_node_id().into()),
            _ => None,
        }
    }

    #[inline]
    pub fn get_bft_leader_id(&self) -> Option<BftLeaderId> {
        match self.block_version() {
            BlockVersion::Ed25519Signed => Some(self.get_cstruct().bft_leader_id().into()),
            _ => None,
        }
    }

    pub fn get_consensus_eval_context(&self) -> ConsensusEvalContext {
        match self.block_version() {
            BlockVersion::KesVrfproof => {
                let nonce = VrfProof(self.get_cstruct().gp_vrf_proof())
                    .to_vrf_proof()
                    .map(|p| leadership::genesis::witness_to_nonce(&p))
                    .expect("internal-error: content_eval_context: vrf proof invalid: shouldn't be trying get an header content application context");
                let node_id = self.get_cstruct().gp_node_id();
                ConsensusEvalContext::Praos {
                    nonce,
                    pool_creator: node_id.into(),
                }
            }
            BlockVersion::Ed25519Signed => ConsensusEvalContext::Bft,
            BlockVersion::Genesis => ConsensusEvalContext::Genesis,
        }
    }

    pub fn get_content_eval_context(&self) -> HeaderContentEvalContext {
        HeaderContentEvalContext {
            block_date: self.block_date(),
            chain_length: self.chain_length(),
            content_hash: self.block_content_hash(),
            consensus_eval_context: self.get_consensus_eval_context(),
            #[cfg(feature = "evm")]
            block_id: self.id(),
        }
    }
}

impl Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hs = self.get_cstruct();
        let mut r = f.debug_struct("Header");
        let r = r
            .field("version", &hs.version())
            .field("content_size", &hs.content_size())
            .field("date", &self.block_date())
            .field("height", &self.chain_length())
            .field("content_hash", &hs.content_hash_ref())
            .field("parent_hash", &hs.parent_hash_ref());
        let r = match self {
            Header::Unsigned(_) => r,
            Header::Bft(_) => r
                .field("bft-leader-id", &hs.bft_leader_id())
                .field("bft-sig", &hs.bft_signature_ref()),
            Header::GenesisPraos(_) => r
                .field("pool-id", &hs.gp_node_id())
                .field("vrf-proof", &hs.gp_vrf_proof_ref())
                .field("kes-sig", &hs.gp_kes_signature_ref()),
        };
        r.field("self_hash", &self.id()).finish()
    }
}

use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError, Serialize},
};

impl Serialize for Header {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.as_slice())
    }
}

impl Deserialize for Header {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let mut buf = Vec::new();
        // TODO: implicitly define size of the Header object in the deserialize function, do not use read_to_end,
        // it narrows the usage of the deserialize trait for the Header struct,
        // which is not obvious from the Deserialze trait description, so leads to mistakes
        codec.read_to_end(&mut buf)?;
        Header::from_slice(buf.as_slice()).map_err(|e| match e {
            HeaderError::InvalidSize => ReadError::NotEnoughBytes(0, 0),
            HeaderError::UnknownVersion => ReadError::UnknownTag(0),
            HeaderError::SizeMismatch { expected, got } => ReadError::SizeTooBig(expected, got),
        })
    }
}
