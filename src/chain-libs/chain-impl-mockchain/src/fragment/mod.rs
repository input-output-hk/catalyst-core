pub mod config;
mod content;
mod raw;

use crate::legacy;
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;

pub use config::ConfigParams;
pub use raw::{FragmentId, FragmentRaw};

pub use content::{BlockContentHash, BlockContentSize, Contents, ContentsBuilder};

use crate::{
    certificate,
    transaction::{NoExtra, Transaction},
    update::{SignedUpdateProposal, SignedUpdateVote},
};

#[cfg(any(test, feature = "property-test-api"))]
pub mod test;

/// Old name for Fragment. (soft) deprecated
pub type Message = Fragment;

/// Old name for FragmentTag. (soft) deprecated
pub(super) type MessageTag = FragmentTag;

/// All possible messages recordable in the content
#[derive(Debug, Clone)]
pub enum Fragment {
    Initial(ConfigParams),
    OldUtxoDeclaration(legacy::UtxoDeclaration),
    Transaction(Transaction<NoExtra>),
    OwnerStakeDelegation(Transaction<certificate::OwnerStakeDelegation>),
    StakeDelegation(Transaction<certificate::StakeDelegation>),
    PoolRegistration(Transaction<certificate::PoolRegistration>),
    PoolRetirement(Transaction<certificate::PoolRetirement>),
    PoolUpdate(Transaction<certificate::PoolUpdate>),
    UpdateProposal(SignedUpdateProposal),
    UpdateVote(SignedUpdateVote),
    VotePlan(Transaction<certificate::VotePlan>),
    VoteCast(Transaction<certificate::VoteCast>),
    VoteTally(Transaction<certificate::VoteTally>),
}

impl PartialEq for Fragment {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}
impl Eq for Fragment {}

/// Tag enumeration of all known fragment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FragmentTag {
    Initial = 0,
    OldUtxoDeclaration = 1,
    Transaction = 2,
    OwnerStakeDelegation = 3,
    StakeDelegation = 4,
    PoolRegistration = 5,
    PoolRetirement = 6,
    PoolUpdate = 7,
    UpdateProposal = 8,
    UpdateVote = 9,
    VotePlan = 10,
    VoteCast = 11,
    VoteTally = 12,
}

impl FragmentTag {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(FragmentTag::Initial),
            1 => Some(FragmentTag::OldUtxoDeclaration),
            2 => Some(FragmentTag::Transaction),
            3 => Some(FragmentTag::OwnerStakeDelegation),
            4 => Some(FragmentTag::StakeDelegation),
            5 => Some(FragmentTag::PoolRegistration),
            6 => Some(FragmentTag::PoolRetirement),
            7 => Some(FragmentTag::PoolUpdate),
            8 => Some(FragmentTag::UpdateProposal),
            9 => Some(FragmentTag::UpdateVote),
            10 => Some(FragmentTag::VotePlan),
            11 => Some(FragmentTag::VoteCast),
            12 => Some(FragmentTag::VoteTally),
            _ => None,
        }
    }
}

impl Fragment {
    /// Return the tag associated with the Message
    pub(super) fn get_tag(&self) -> MessageTag {
        match self {
            Fragment::Initial(_) => FragmentTag::Initial,
            Fragment::OldUtxoDeclaration(_) => FragmentTag::OldUtxoDeclaration,
            Fragment::Transaction(_) => FragmentTag::Transaction,
            Fragment::OwnerStakeDelegation(_) => FragmentTag::OwnerStakeDelegation,
            Fragment::StakeDelegation(_) => FragmentTag::StakeDelegation,
            Fragment::PoolRegistration(_) => FragmentTag::PoolRegistration,
            Fragment::PoolRetirement(_) => FragmentTag::PoolRetirement,
            Fragment::PoolUpdate(_) => FragmentTag::PoolUpdate,
            Fragment::UpdateProposal(_) => FragmentTag::UpdateProposal,
            Fragment::UpdateVote(_) => FragmentTag::UpdateVote,
            Fragment::VotePlan(_) => FragmentTag::VotePlan,
            Fragment::VoteCast(_) => FragmentTag::VoteCast,
            Fragment::VoteTally(_) => FragmentTag::VoteTally,
        }
    }

    /// Get the serialized representation of this message
    pub fn to_raw(&self) -> FragmentRaw {
        use chain_core::packer::*;
        use chain_core::property::Serialize;
        let v = Vec::new();
        let mut codec = Codec::new(v);
        codec.put_u8(0).unwrap();
        codec.put_u8(self.get_tag() as u8).unwrap();
        match self {
            Fragment::Initial(i) => i.serialize(&mut codec).unwrap(),
            Fragment::OldUtxoDeclaration(s) => s.serialize(&mut codec).unwrap(),
            Fragment::Transaction(signed) => signed.serialize(&mut codec).unwrap(),
            Fragment::OwnerStakeDelegation(od) => od.serialize(&mut codec).unwrap(),
            Fragment::StakeDelegation(od) => od.serialize(&mut codec).unwrap(),
            Fragment::PoolRegistration(atx) => atx.serialize(&mut codec).unwrap(),
            Fragment::PoolRetirement(pm) => pm.serialize(&mut codec).unwrap(),
            Fragment::PoolUpdate(pm) => pm.serialize(&mut codec).unwrap(),
            Fragment::UpdateProposal(proposal) => proposal.serialize(&mut codec).unwrap(),
            Fragment::UpdateVote(vote) => vote.serialize(&mut codec).unwrap(),
            Fragment::VotePlan(vote_plan) => vote_plan.serialize(&mut codec).unwrap(),
            Fragment::VoteCast(vote_plan) => vote_plan.serialize(&mut codec).unwrap(),
            Fragment::VoteTally(vote_tally) => vote_tally.serialize(&mut codec).unwrap(),
        }
        FragmentRaw(codec.into_inner())
    }

    pub fn from_raw(raw: &FragmentRaw) -> Result<Self, ReadError> {
        let mut buf = ReadBuf::from(raw.as_ref());
        Fragment::read(&mut buf)
    }

    /// The ID of a message is a hash of its serialization *without* the size.
    pub fn hash(&self) -> FragmentId {
        self.to_raw().id()
    }
}

impl Readable for Fragment {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let padding_tag = buf.get_u8()?;
        if padding_tag != 0 {
            return Err(ReadError::StructureInvalid(format!(
                "fragment padding tag expected at 0 but got {}",
                padding_tag
            )));
        }

        let tag = buf.get_u8()?;
        match FragmentTag::from_u8(tag) {
            Some(FragmentTag::Initial) => ConfigParams::read(buf).map(Fragment::Initial),
            Some(FragmentTag::OldUtxoDeclaration) => {
                legacy::UtxoDeclaration::read(buf).map(Fragment::OldUtxoDeclaration)
            }
            Some(FragmentTag::Transaction) => Transaction::read(buf).map(Fragment::Transaction),
            Some(FragmentTag::OwnerStakeDelegation) => {
                Transaction::read(buf).map(Fragment::OwnerStakeDelegation)
            }
            Some(FragmentTag::StakeDelegation) => {
                Transaction::read(buf).map(Fragment::StakeDelegation)
            }
            Some(FragmentTag::PoolRegistration) => {
                Transaction::read(buf).map(Fragment::PoolRegistration)
            }
            Some(FragmentTag::PoolRetirement) => {
                Transaction::read(buf).map(Fragment::PoolRetirement)
            }
            Some(FragmentTag::PoolUpdate) => Transaction::read(buf).map(Fragment::PoolUpdate),
            Some(FragmentTag::UpdateProposal) => {
                SignedUpdateProposal::read(buf).map(Fragment::UpdateProposal)
            }
            Some(FragmentTag::UpdateVote) => SignedUpdateVote::read(buf).map(Fragment::UpdateVote),
            Some(FragmentTag::VotePlan) => Transaction::read(buf).map(Fragment::VotePlan),
            Some(FragmentTag::VoteCast) => Transaction::read(buf).map(Fragment::VoteCast),
            Some(FragmentTag::VoteTally) => Transaction::read(buf).map(Fragment::VoteTally),
            None => Err(ReadError::UnknownTag(tag as u32)),
        }
    }
}

impl property::Serialize for Fragment {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        self.to_raw().serialize(writer)
    }
}

impl property::Deserialize for Fragment {
    type Error = std::io::Error;
    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        let raw = FragmentRaw::deserialize(reader)?;
        Fragment::from_raw(&raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
    }
}

impl property::Fragment for Fragment {
    type Id = FragmentId;

    /// The ID of a fragment is a hash of its serialization *without* the size.
    fn id(&self) -> Self::Id {
        self.hash()
    }
}
