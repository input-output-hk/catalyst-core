use crate::{
    block::BlockDate,
    certificate::CertificateSlice,
    ledger::governance::{Governance, ParametersGovernanceAction, TreasuryGovernanceAction},
    tokens::identifier::TokenIdentifier,
    transaction::{
        Payload, PayloadAuthData, PayloadData, PayloadSlice, SingleAccountBindingSignature,
        TransactionBindingAuthData,
    },
    vote,
};
use chain_core::{
    packer::Codec,
    property::{Deserialize, DeserializeFromSlice, ReadError, Serialize, WriteError},
};
use chain_crypto::{digest::DigestOf, Blake2b256, Verification};
use chain_vote::MemberPublicKey;
use std::ops::Deref;
use typed_bytes::{ByteArray, ByteBuilder};

/// abstract tag type to represent an external document, whatever it may be
pub struct ExternalProposalDocument;

/// the identifier of the external proposal is the Blake2b 256 bits of the
/// external proposal document hash.
///
pub type ExternalProposalId = DigestOf<Blake2b256, ExternalProposalDocument>;

/// the vote plan identifier on the blockchain
pub type VotePlanId = DigestOf<Blake2b256, VotePlan>;

/// a vote plan for the voting system
///
/// A vote plan defines what is being voted, for how long and how long
/// is the committee supposed to reveal the results.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VotePlan {
    /// the vote start validity
    vote_start: BlockDate,
    /// the duration within which it is possible to vote for one of the proposals
    /// of this voting plan.
    vote_end: BlockDate,
    /// the committee duration is the time allocated to the committee to open
    /// the ballots and publish the results on chain
    committee_end: BlockDate,
    /// the proposals to vote for
    proposals: Proposals,
    /// vote payload type
    payload_type: vote::PayloadType,
    /// encrypting votes public keys
    committee_public_keys: Vec<chain_vote::MemberPublicKey>,
    /// voting token used for weigthing the votes for any proposal in this voteplan
    voting_token: TokenIdentifier,
}

#[derive(Debug, Clone)]
pub struct VotePlanProof {
    pub id: vote::CommitteeId,
    pub signature: SingleAccountBindingSignature,
}

/// this is the action that will result of the vote
///
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteAction {
    /// the action if off chain or not relevant to the blockchain
    OffChain,
    /// control the treasury
    Treasury { action: TreasuryGovernanceAction },
    /// control the parameters
    Parameters { action: ParametersGovernanceAction },
}

/// a collection of proposals
///
/// there may not be more than 255 proposal
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Proposals {
    proposals: Vec<Proposal>,
}

/// a proposal with the associated external proposal identifier
/// which leads to retrieving data from outside of the blockchain
/// with its unique identifier and the funding plan required
/// for the proposal to be operated.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    external_id: ExternalProposalId,
    options: vote::Options,
    action: VoteAction,
}

#[must_use = "Adding a proposal may fail"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushProposal {
    Success,
    Full { proposal: Proposal },
}

impl Proposal {
    pub fn new(
        external_id: ExternalProposalId,
        options: vote::Options,
        action: VoteAction,
    ) -> Self {
        Self {
            external_id,
            options,
            action,
        }
    }

    pub fn check_governance(&self, governance: &Governance) -> bool {
        let criteria = match self.action() {
            VoteAction::OffChain => {
                // OffChain passes acceptance as it does not require governance
                return true;
            }
            VoteAction::Parameters { action } => governance
                .parameters
                .acceptance_criteria_for(action.to_type()),
            VoteAction::Treasury { action } => governance
                .treasury
                .acceptance_criteria_for(action.to_type()),
        };

        criteria.options == self.options
    }

    pub fn external_id(&self) -> &ExternalProposalId {
        &self.external_id
    }

    pub fn options(&self) -> &vote::Options {
        &self.options
    }

    pub fn action(&self) -> &VoteAction {
        &self.action
    }

    fn serialize_in(&self, bb: ByteBuilder<VotePlan>) -> ByteBuilder<VotePlan> {
        bb.bytes(self.external_id.as_ref())
            .u8(self.options.as_byte())
            .sub(|bb| self.action.serialize_in(bb))
    }
}

impl VoteAction {
    fn serialize_in(&self, bb: ByteBuilder<VotePlan>) -> ByteBuilder<VotePlan> {
        match self {
            Self::OffChain => bb.u8(0),
            Self::Treasury { action } => bb.u8(1).sub(|bb| action.serialize_in(bb)),
            Self::Parameters { action } => bb.u8(2).sub(|bb| action.serialize_in(bb)),
        }
    }
}

impl Proposals {
    /// the maximum number of proposals to push in `Proposals`
    pub const MAX_LEN: usize = 255;

    pub fn new() -> Proposals {
        Self {
            proposals: Vec::with_capacity(12),
        }
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.len() >= Self::MAX_LEN
    }

    /// attempt to add a new proposal in the proposal collection
    ///
    pub fn push(&mut self, proposal: Proposal) -> PushProposal {
        if self.full() {
            PushProposal::Full { proposal }
        } else {
            self.proposals.push(proposal);
            PushProposal::Success
        }
    }

    pub fn proposals(&self) -> &Vec<Proposal> {
        &self.proposals
    }
}

impl VotePlan {
    pub fn new(
        vote_start: BlockDate,
        vote_end: BlockDate,
        committee_end: BlockDate,
        proposals: Proposals,
        payload_type: vote::PayloadType,
        committee_public_keys: Vec<chain_vote::MemberPublicKey>,
        voting_token: TokenIdentifier,
    ) -> Self {
        Self {
            vote_start,
            vote_end,
            committee_end,
            proposals,
            payload_type,
            committee_public_keys,
            voting_token,
        }
    }

    pub fn check_governance(&self, governance: &Governance) -> bool {
        self.proposals()
            .iter()
            .all(|proposal| proposal.check_governance(governance))
    }

    pub fn is_governance(&self) -> bool {
        self.proposals()
            .iter()
            .any(|proposal| match proposal.action() {
                VoteAction::OffChain => false,
                VoteAction::Parameters { .. } | VoteAction::Treasury { .. } => true,
            })
    }

    pub fn vote_start(&self) -> BlockDate {
        self.vote_start
    }

    pub fn vote_end(&self) -> BlockDate {
        self.vote_end
    }

    pub fn committee_start(&self) -> BlockDate {
        self.vote_end
    }

    pub fn committee_end(&self) -> BlockDate {
        self.committee_end
    }

    /// access the proposals associated to this voting plan
    pub fn proposals(&self) -> &Proposals {
        &self.proposals
    }

    pub fn proposals_mut(&mut self) -> &mut Proposals {
        &mut self.proposals
    }

    pub fn payload_type(&self) -> vote::PayloadType {
        self.payload_type
    }

    pub fn committee_public_keys(&self) -> &[chain_vote::MemberPublicKey] {
        &self.committee_public_keys
    }

    pub fn voting_token(&self) -> &TokenIdentifier {
        &self.voting_token
    }

    #[inline]
    pub fn vote_started(&self, date: BlockDate) -> bool {
        self.vote_start <= date
    }

    #[inline]
    pub fn vote_finished(&self, date: BlockDate) -> bool {
        self.vote_end <= date
    }

    /// tells if it is possible to vote at the given date
    ///
    /// `[vote_start..vote_end[`: from the start date (included) to
    /// the end (not included).
    #[inline]
    pub fn can_vote(&self, date: BlockDate) -> bool {
        self.vote_started(date) && !self.vote_finished(date)
    }

    #[inline]
    pub fn committee_started(&self, date: BlockDate) -> bool {
        self.vote_end <= date
    }

    #[inline]
    pub fn committee_finished(&self, date: BlockDate) -> bool {
        self.committee_end <= date
    }

    /// tells if it is possible to do the committee operations at the given date
    ///
    /// `[vote_end..committee_end[` from the end of the vote date (included) to the
    /// end of the committee (not included).
    ///
    #[inline]
    pub fn committee_time(&self, date: BlockDate) -> bool {
        self.committee_started(date) && !self.committee_finished(date)
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.u32(self.vote_start.epoch)
            .u32(self.vote_start.slot_id)
            .u32(self.vote_end.epoch)
            .u32(self.vote_end.slot_id)
            .u32(self.committee_end.epoch)
            .u32(self.committee_end.slot_id)
            .u8(self.payload_type as u8)
            .iter8(&mut self.proposals.iter(), |bb, proposal| {
                proposal.serialize_in(bb)
            })
            .iter8(self.committee_public_keys.iter(), |bb, key| {
                bb.bytes(key.to_bytes().as_ref())
            })
            .bytes(self.voting_token.bytes().as_ref())
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }

    /// build the identifier of the vote plan
    ///
    /// this is not a very efficient function so it is better not
    /// to call it in tight loop
    pub fn to_id(&self) -> VotePlanId {
        let ba = self.serialize();
        DigestOf::digest_byteslice(&ba.as_byteslice())
    }
}

impl VotePlanProof {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.bytes(self.id.as_ref()).bytes(self.signature.as_ref())
    }

    pub fn verify<'a>(&self, verify_data: &TransactionBindingAuthData<'a>) -> Verification {
        let pk = self.id.public_key();
        self.signature.verify_slice(&pk, verify_data)
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for VotePlan {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true;
    type Auth = VotePlanProof;

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

    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

/* Deref ******************************************************************** */

impl Deref for Proposals {
    type Target = <Vec<Proposal> as Deref>::Target;
    fn deref(&self) -> &Self::Target {
        self.proposals.deref()
    }
}

/* Ser/De ******************************************************************* */

impl Serialize for VotePlan {
    fn serialized_size(&self) -> usize {
        self.serialize().as_slice().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.serialize().as_slice())
    }
}

impl DeserializeFromSlice for VotePlanProof {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let id = vote::CommitteeId::deserialize_from_slice(codec)?;
        let signature = SingleAccountBindingSignature::deserialize_from_slice(codec)?;
        Ok(Self { id, signature })
    }
}

impl Deserialize for VoteAction {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        match codec.get_u8()? {
            0 => Ok(Self::OffChain),
            1 => {
                TreasuryGovernanceAction::deserialize(codec).map(|action| Self::Treasury { action })
            }
            2 => ParametersGovernanceAction::deserialize(codec)
                .map(|action| Self::Parameters { action }),
            t => Err(ReadError::UnknownTag(t as u32)),
        }
    }
}

impl DeserializeFromSlice for VotePlan {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let vote_start = BlockDate {
            epoch: codec.get_be_u32()?,
            slot_id: codec.get_be_u32()?,
        };
        let vote_end = BlockDate {
            epoch: codec.get_be_u32()?,
            slot_id: codec.get_be_u32()?,
        };
        let committee_end = BlockDate {
            epoch: codec.get_be_u32()?,
            slot_id: codec.get_be_u32()?,
        };

        let payload_type = codec
            .get_u8()?
            .try_into()
            .map_err(|e: vote::TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        let proposal_size = codec.get_u8()? as usize;
        let mut proposals = Proposals {
            proposals: Vec::with_capacity(proposal_size),
        };
        for _ in 0..proposal_size {
            let external_id = <[u8; 32]>::deserialize(codec)?.into();
            let options = codec.get_u8().and_then(|num_choices| {
                vote::Options::new_length(num_choices)
                    .map_err(|e| ReadError::StructureInvalid(e.to_string()))
            })?;
            let action = VoteAction::deserialize(codec)?;

            let proposal = Proposal {
                external_id,
                options,
                action,
            };

            proposals.proposals.push(proposal);
        }

        let member_keys_len = codec.get_u8()?;
        let mut committee_public_keys = Vec::new();
        for _ in 0..member_keys_len {
            let key_buf = codec.get_slice(MemberPublicKey::BYTES_LEN)?;
            committee_public_keys.push(MemberPublicKey::from_bytes(key_buf).ok_or_else(|| {
                ReadError::StructureInvalid("invalid public key format".to_string())
            })?);
        }

        let voting_token = TokenIdentifier::deserialize(codec)?;

        Ok(Self {
            vote_start,
            vote_end,
            committee_end,
            proposals,
            payload_type,
            committee_public_keys,
            voting_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockDate;
    use crate::testing::VoteTestGen;
    use crate::tokens::name::{TokenName, TOKEN_NAME_MAX_SIZE};
    use crate::tokens::policy_hash::{PolicyHash, POLICY_HASH_SIZE};
    use chain_core::property::BlockDate as BlockDateProp;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn serialize_deserialize(vote_plan: VotePlan) -> bool {
        let serialized = vote_plan.serialize();

        let result = VotePlan::deserialize_from_slice(&mut Codec::new(serialized.as_ref()));

        let decoded = result.expect("can decode encoded vote plan");

        decoded == vote_plan
    }

    #[test]
    pub fn proposals_are_full() {
        let mut proposals = VoteTestGen::proposals(Proposals::MAX_LEN);
        assert!(proposals.full());
        let proposal = VoteTestGen::proposal();
        let expected = PushProposal::Full {
            proposal: proposal.clone(),
        };
        assert_eq!(expected, proposals.push(proposal));
    }

    #[test]
    pub fn vote_and_committee_dates_are_left_inclusive() {
        let vote_start = BlockDate::first();
        let vote_end = vote_start.next_epoch();
        let committee_finished = vote_end.next_epoch();
        let vote_plan = VotePlan::new(
            vote_start,
            vote_end,
            committee_finished,
            VoteTestGen::proposals(1),
            vote::PayloadType::Public,
            Vec::new(),
            TokenIdentifier {
                policy_hash: PolicyHash::from([0u8; POLICY_HASH_SIZE]),
                token_name: TokenName::try_from(vec![0u8; TOKEN_NAME_MAX_SIZE]).unwrap(),
            },
        );

        assert!(vote_plan.vote_started(vote_start));
        assert!(vote_plan.vote_finished(vote_end));
        assert!(vote_plan.committee_started(vote_end));
        assert!(vote_plan.committee_finished(committee_finished));
    }

    #[test]
    pub fn correct_vote_plan_timeline() {
        let vote_start = BlockDate::from_epoch_slot_id(1, 0);
        let vote_end = vote_start.next_epoch();
        let committee_finished = vote_end.next_epoch();
        let vote_plan = VotePlan::new(
            vote_start,
            vote_end,
            committee_finished,
            VoteTestGen::proposals(1),
            vote::PayloadType::Public,
            Vec::new(),
            TokenIdentifier {
                policy_hash: PolicyHash::from([0u8; POLICY_HASH_SIZE]),
                token_name: TokenName::try_from(vec![0u8; TOKEN_NAME_MAX_SIZE]).unwrap(),
            },
        );

        let before_voting = BlockDate::from_epoch_slot_id(0, 10);
        let voting_date = BlockDate::from_epoch_slot_id(1, 10);
        let committee_time = BlockDate::from_epoch_slot_id(2, 10);
        let after_committee_time = BlockDate::from_epoch_slot_id(3, 10);

        assert!(!vote_plan.can_vote(before_voting));
        assert!(!vote_plan.committee_time(before_voting));

        assert!(vote_plan.can_vote(voting_date));
        assert!(!vote_plan.committee_time(voting_date));

        assert!(!vote_plan.can_vote(committee_time));
        assert!(vote_plan.committee_time(committee_time));

        assert!(!vote_plan.can_vote(after_committee_time));
        assert!(!vote_plan.committee_time(after_committee_time));
    }

    #[test]
    pub fn inverted_vote_plan_timeline() {
        let vote_start = BlockDate::from_epoch_slot_id(1, 0);
        let vote_end = vote_start.next_epoch();
        let committee_finished = vote_end.next_epoch();
        let vote_plan = VotePlan::new(
            committee_finished,
            vote_end,
            vote_start,
            VoteTestGen::proposals(1),
            vote::PayloadType::Public,
            Vec::new(),
            TokenIdentifier {
                policy_hash: PolicyHash::from([0u8; POLICY_HASH_SIZE]),
                token_name: TokenName::try_from(vec![0u8; TOKEN_NAME_MAX_SIZE]).unwrap(),
            },
        );

        let before_voting = BlockDate::from_epoch_slot_id(0, 10);
        let voting_date = BlockDate::from_epoch_slot_id(1, 10);
        let committee_time = BlockDate::from_epoch_slot_id(2, 10);
        let after_committee_time = BlockDate::from_epoch_slot_id(3, 10);

        assert!(!vote_plan.can_vote(before_voting));
        assert!(!vote_plan.committee_time(before_voting));

        assert!(!vote_plan.can_vote(voting_date));
        assert!(!vote_plan.committee_time(voting_date));

        assert!(!vote_plan.can_vote(committee_time));
        assert!(!vote_plan.committee_time(committee_time));

        assert!(!vote_plan.can_vote(after_committee_time));
        assert!(!vote_plan.committee_time(after_committee_time));
    }
}
