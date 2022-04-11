pub mod config;
mod content;

use crate::key::Hash;
use crate::legacy;
use chain_core::{
    packer::Codec,
    property::{self, Deserialize, DeserializeFromSlice, ReadError, Serialize, WriteError},
};

pub use config::ConfigParams;

pub use content::{BlockContentHash, BlockContentSize, Contents, ContentsBuilder};

use crate::{
    certificate,
    evm::EvmTransaction,
    transaction::{NoExtra, Transaction},
};

pub type FragmentId = Hash;

#[cfg(any(test, feature = "property-test-api"))]
pub mod test;

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
    UpdateProposal(Transaction<certificate::UpdateProposal>),
    UpdateVote(Transaction<certificate::UpdateVote>),
    VotePlan(Transaction<certificate::VotePlan>),
    VoteCast(Transaction<certificate::VoteCast>),
    VoteTally(Transaction<certificate::VoteTally>),
    MintToken(Transaction<certificate::MintToken>),
    Evm(Transaction<EvmTransaction>),
    EvmMapping(Transaction<certificate::EvmMapping>),
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
    MintToken = 13,
    Evm = 14,
    EvmMapping = 15,
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
            13 => Some(FragmentTag::MintToken),
            14 => Some(FragmentTag::Evm),
            _ => None,
        }
    }
}

impl Fragment {
    /// Return the tag associated with the Message
    pub(super) fn get_tag(&self) -> FragmentTag {
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
            Fragment::MintToken(_) => FragmentTag::MintToken,
            Fragment::Evm(_) => FragmentTag::Evm,
            Fragment::EvmMapping(_) => FragmentTag::EvmMapping,
        }
    }

    /// The ID of a message is a hash of its serialization *without* the size.
    pub fn hash(&self) -> FragmentId {
        FragmentId::hash_bytes(self.serialize_as_vec().unwrap().as_slice())
    }
}

impl Deserialize for Fragment {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let size = codec.get_be_u32()? as usize;
        let bytes = codec.get_bytes(size)?;

        let mut codec = Codec::new(bytes.as_slice());

        let padding_tag = codec.get_u8()?;
        if padding_tag != 0 {
            return Err(ReadError::StructureInvalid(format!(
                "fragment padding tag expected at 0 but got {}",
                padding_tag
            )));
        }

        let tag = codec.get_u8()?;
        match FragmentTag::from_u8(tag) {
            Some(FragmentTag::Initial) => {
                ConfigParams::deserialize_from_slice(&mut codec).map(Fragment::Initial)
            }
            Some(FragmentTag::OldUtxoDeclaration) => {
                legacy::UtxoDeclaration::deserialize_from_slice(&mut codec)
                    .map(Fragment::OldUtxoDeclaration)
            }
            Some(FragmentTag::Transaction) => {
                Transaction::deserialize(&mut codec).map(Fragment::Transaction)
            }
            Some(FragmentTag::OwnerStakeDelegation) => {
                Transaction::deserialize(&mut codec).map(Fragment::OwnerStakeDelegation)
            }
            Some(FragmentTag::StakeDelegation) => {
                Transaction::deserialize(&mut codec).map(Fragment::StakeDelegation)
            }
            Some(FragmentTag::PoolRegistration) => {
                Transaction::deserialize(&mut codec).map(Fragment::PoolRegistration)
            }
            Some(FragmentTag::PoolRetirement) => {
                Transaction::deserialize(&mut codec).map(Fragment::PoolRetirement)
            }
            Some(FragmentTag::PoolUpdate) => {
                Transaction::deserialize(&mut codec).map(Fragment::PoolUpdate)
            }
            Some(FragmentTag::UpdateProposal) => {
                Transaction::deserialize(&mut codec).map(Fragment::UpdateProposal)
            }
            Some(FragmentTag::UpdateVote) => {
                Transaction::deserialize(&mut codec).map(Fragment::UpdateVote)
            }
            Some(FragmentTag::VotePlan) => {
                Transaction::deserialize(&mut codec).map(Fragment::VotePlan)
            }
            Some(FragmentTag::VoteCast) => {
                Transaction::deserialize(&mut codec).map(Fragment::VoteCast)
            }
            Some(FragmentTag::VoteTally) => {
                Transaction::deserialize(&mut codec).map(Fragment::VoteTally)
            }
            Some(FragmentTag::MintToken) => {
                Transaction::deserialize(&mut codec).map(Fragment::MintToken)
            }
            Some(FragmentTag::Evm) => Transaction::deserialize(&mut codec).map(Fragment::Evm),
            Some(FragmentTag::EvmMapping) => {
                Transaction::deserialize(&mut codec).map(Fragment::EvmMapping)
            }
            None => Err(ReadError::UnknownTag(tag as u32)),
        }
    }
}

impl Serialize for Fragment {
    fn serialized_size(&self) -> usize {
        Codec::u8_size()
            + Codec::u8_size()
            + match self {
                Fragment::Initial(i) => i.serialized_size(),
                Fragment::OldUtxoDeclaration(s) => s.serialized_size(),
                Fragment::Transaction(signed) => signed.serialized_size(),
                Fragment::OwnerStakeDelegation(od) => od.serialized_size(),
                Fragment::StakeDelegation(od) => od.serialized_size(),
                Fragment::PoolRegistration(atx) => atx.serialized_size(),
                Fragment::PoolRetirement(pm) => pm.serialized_size(),
                Fragment::PoolUpdate(pm) => pm.serialized_size(),
                Fragment::UpdateProposal(proposal) => proposal.serialized_size(),
                Fragment::UpdateVote(vote) => vote.serialized_size(),
                Fragment::VotePlan(vote_plan) => vote_plan.serialized_size(),
                Fragment::VoteCast(vote_plan) => vote_plan.serialized_size(),
                Fragment::VoteTally(vote_tally) => vote_tally.serialized_size(),
                Fragment::MintToken(mint_token) => mint_token.serialized_size(),
                Fragment::Evm(deployment) => deployment.serialized_size(),
                Fragment::EvmMapping(evm_mapping) => evm_mapping.serialized_size(),
            }
            + Codec::u32_size()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        let mut tmp = Codec::new(Vec::new());
        tmp.put_u8(0).unwrap();
        tmp.put_u8(self.get_tag() as u8).unwrap();
        match self {
            Fragment::Initial(i) => i.serialize(&mut tmp)?,
            Fragment::OldUtxoDeclaration(s) => s.serialize(&mut tmp)?,
            Fragment::Transaction(signed) => signed.serialize(&mut tmp)?,
            Fragment::OwnerStakeDelegation(od) => od.serialize(&mut tmp)?,
            Fragment::StakeDelegation(od) => od.serialize(&mut tmp)?,
            Fragment::PoolRegistration(atx) => atx.serialize(&mut tmp)?,
            Fragment::PoolRetirement(pm) => pm.serialize(&mut tmp)?,
            Fragment::PoolUpdate(pm) => pm.serialize(&mut tmp)?,
            Fragment::UpdateProposal(proposal) => proposal.serialize(&mut tmp)?,
            Fragment::UpdateVote(vote) => vote.serialize(&mut tmp)?,
            Fragment::VotePlan(vote_plan) => vote_plan.serialize(&mut tmp)?,
            Fragment::VoteCast(vote_plan) => vote_plan.serialize(&mut tmp)?,
            Fragment::VoteTally(vote_tally) => vote_tally.serialize(&mut tmp)?,
            Fragment::MintToken(mint_token) => mint_token.serialize(&mut tmp)?,
            Fragment::Evm(deployment) => deployment.serialize(&mut tmp)?,
            Fragment::EvmMapping(evm_mapping) => evm_mapping.serialize(&mut tmp)?,
        };
        let bytes = tmp.into_inner();
        codec.put_be_u32(bytes.len() as u32)?;
        codec.put_bytes(bytes.as_slice())
    }
}

impl property::Fragment for Fragment {
    type Id = FragmentId;

    /// The ID of a fragment is a hash of its serialization *without* the size.
    fn id(&self) -> Self::Id {
        self.hash()
    }
}
