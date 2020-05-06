use crate::{
    block::BlockDate,
    certificate::CertificateSlice,
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
};
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use chain_crypto::{digest::DigestOf, Blake2b256};
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
}

/// a collection of proposals
///
/// there may not be more than 255 proposal
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Proposals {
    proposals: Vec<Proposal>,
}

/// options for the vote
///
/// currently this is a 4bits structure, allowing up to 16 choices
/// however we may allow more complex object to be set in
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VoteOptions {
    num_choices: u8,
}

/// a proposal with the associated external proposal identifier
/// which leads to retrieving data from outside of the blockchain
/// with its unique identifier and the funding plan required
/// for the proposal to be operated.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    external_id: ExternalProposalId,
    options: VoteOptions,
}

#[must_use = "Adding a proposal may fail"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushProposal {
    Success,
    Full { proposal: Proposal },
}

impl VoteOptions {
    const NUM_CHOICES_MAX: u8 = 0b0000_1111;

    pub fn new_length(num_choices: u8) -> Self {
        Self {
            num_choices: num_choices & Self::NUM_CHOICES_MAX,
        }
    }

    pub fn as_byte(self) -> u8 {
        self.num_choices
    }
}

impl Proposal {
    pub fn new(external_id: ExternalProposalId, options: VoteOptions) -> Self {
        Self {
            external_id,
            options,
        }
    }

    pub fn external_id(&self) -> &ExternalProposalId {
        &self.external_id
    }

    pub fn options(&self) -> &VoteOptions {
        &self.options
    }

    fn serialize_in(&self, bb: ByteBuilder<VotePlan>) -> ByteBuilder<VotePlan> {
        bb.bytes(self.external_id.as_ref())
            .u8(self.options.num_choices)
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
}

impl VotePlan {
    pub fn new(
        vote_start: BlockDate,
        vote_end: BlockDate,
        committee_end: BlockDate,
        proposals: Proposals,
    ) -> Self {
        Self {
            vote_start,
            vote_end,
            committee_end,
            proposals,
        }
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
            .iter8(&mut self.proposals.iter(), |bb, proposal| {
                proposal.serialize_in(bb)
            })
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

/* Auth/Payload ************************************************************* */

impl Payload for VotePlan {
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

    fn to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
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

impl property::Serialize for VotePlan {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.serialize().as_slice())?;
        Ok(())
    }
}

impl Readable for VotePlan {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let vote_start = BlockDate {
            epoch: buf.get_u32()?,
            slot_id: buf.get_u32()?,
        };
        let vote_end = BlockDate {
            epoch: buf.get_u32()?,
            slot_id: buf.get_u32()?,
        };
        let committee_end = BlockDate {
            epoch: buf.get_u32()?,
            slot_id: buf.get_u32()?,
        };

        let proposal_size = buf.get_u8()? as usize;
        let mut proposals = Proposals {
            proposals: Vec::with_capacity(proposal_size),
        };
        for _ in 0..proposal_size {
            let external_id = <[u8; 32]>::read(buf)?.into();
            let options = buf.get_u8().map(VoteOptions::new_length)?;

            let proposal = Proposal {
                external_id,
                options,
            };

            proposals.proposals.push(proposal);
        }

        Ok(Self {
            vote_start,
            vote_end,
            committee_end,
            proposals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockDate;
    use crate::testing::VoteTestGen;
    use chain_core::property::BlockDate as BlockDateProp;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn serialize_deserialize(vote_plan: VotePlan) -> bool {
        let serialized = vote_plan.serialize();

        let mut buf = ReadBuf::from(serialized.as_ref());
        let result = VotePlan::read(&mut buf);

        let decoded = result.expect("can decode encoded vote plan");

        decoded == vote_plan
    }

    #[quickcheck]
    pub fn vote_options_max(count: u8) -> TestResult {
        TestResult::from_bool(VoteOptions::new_length(count).as_byte() == count % 16)
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
        );

        let before_voting = BlockDate::from_epoch_slot_id(0, 10);
        let voting_date = BlockDate::from_epoch_slot_id(1, 10);
        let committe_time = BlockDate::from_epoch_slot_id(2, 10);
        let after_committe_time = BlockDate::from_epoch_slot_id(3, 10);

        assert!(!vote_plan.can_vote(before_voting));
        assert!(!vote_plan.committee_time(before_voting));

        assert!(vote_plan.can_vote(voting_date));
        assert!(!vote_plan.committee_time(voting_date));

        assert!(!vote_plan.can_vote(committe_time));
        assert!(vote_plan.committee_time(committe_time));

        assert!(!vote_plan.can_vote(after_committe_time));
        assert!(!vote_plan.committee_time(after_committe_time));
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
        );

        let before_voting = BlockDate::from_epoch_slot_id(0, 10);
        let voting_date = BlockDate::from_epoch_slot_id(1, 10);
        let committe_time = BlockDate::from_epoch_slot_id(2, 10);
        let after_committe_time = BlockDate::from_epoch_slot_id(3, 10);

        assert!(!vote_plan.can_vote(before_voting));
        assert!(!vote_plan.committee_time(before_voting));

        assert!(!vote_plan.can_vote(voting_date));
        assert!(!vote_plan.committee_time(voting_date));

        assert!(!vote_plan.can_vote(committe_time));
        assert!(!vote_plan.committee_time(committe_time));

        assert!(!vote_plan.can_vote(after_committe_time));
        assert!(!vote_plan.committee_time(after_committe_time));
    }
}
