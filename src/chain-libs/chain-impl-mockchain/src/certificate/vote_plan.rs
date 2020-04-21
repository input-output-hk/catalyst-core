use crate::value::Value;
use chain_addr::Address;
use chain_crypto::{digest::DigestOf, Blake2b256};
use chain_time::{DurationSeconds, TimeOffsetSeconds};
use std::ops::Deref;

/// abstract tag type to represent an external document, whatever it may be
pub struct ExternalProposalDocument;

/// the identifier of the external proposal is the Blake2b 256 bits of the
/// external proposal document hash.
///
pub type ExternalProposalId = DigestOf<Blake2b256, ExternalProposalDocument>;

/// a vote plan for the voting system
///
/// A vote plan defines what is being voted, for how long and how long
/// is the committee supposed to reveal the results.
///
#[derive(Debug)]
pub struct VotePlan {
    /// the vote start validity
    vote_start: TimeOffsetSeconds,
    /// the duration within which it is possible to vote for one of the proposals
    /// of this voting plan.
    vote_duration: DurationSeconds,
    /// the committee duration is the time allocated to the committee to open
    /// the ballots and publish the results on chain
    committee_duration: DurationSeconds,
    /// the proposals to vote for
    proposals: Proposals,
}

/// a collection of proposals
///
/// there may not be more than 255 proposal
#[derive(Debug)]
pub struct Proposals {
    proposals: Vec<Proposal>,
}

/// a proposal with the associated external proposal identifier
/// which leads to retrieving data from outside of the blockchain
/// with its unique identifier and the funding plan required
/// for the proposal to be operated.
///
#[derive(Debug)]
pub struct Proposal {
    external_id: ExternalProposalId,
    funding_plan: FundingPlan,
}

#[derive(Debug)]
pub enum FundingPlan {
    /// may the associated proposal be voted for, the funding
    /// will be paid upfront to the given address.
    Upfront {
        /// the value required to pay upfront
        value: Value,
        /// the address to deposit the funds to
        address: Address,
    },
}

#[must_use = "Adding a proposal may fail"]
pub enum PushProposal {
    Success,
    Full { proposal: Proposal },
}

impl Proposal {
    pub fn new(external_id: ExternalProposalId, funding_plan: FundingPlan) -> Self {
        Self {
            external_id,
            funding_plan,
        }
    }

    pub fn external_id(&self) -> &ExternalProposalId {
        &self.external_id
    }

    pub fn funding_plan(&self) -> &FundingPlan {
        &self.funding_plan
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
    /// access the proposals associated to this voting plan
    pub fn proposals(&self) -> &Proposals {
        &self.proposals
    }

    pub fn proposals_mut(&mut self) -> &mut Proposals {
        &mut self.proposals
    }

    #[inline]
    fn vote_end_offset(&self) -> TimeOffsetSeconds {
        let start: u64 = self.vote_start.into();
        let duration: u64 = self.vote_duration.into();
        let end = start + duration;
        let end: DurationSeconds = end.into();
        end.into()
    }

    #[inline]
    fn committee_end_offset(&self) -> TimeOffsetSeconds {
        let start: u64 = self.vote_end_offset().into();
        let duration: u64 = self.committee_duration.into();
        let end = start + duration;
        let end: DurationSeconds = end.into();
        end.into()
    }

    #[inline]
    pub fn vote_started(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn vote_finished(&self) -> bool {
        todo!()
    }

    pub fn can_vote(&self) -> bool {
        self.vote_started() && !self.vote_finished()
    }

    #[inline]
    pub fn committee_started(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn committee_finished(&self) -> bool {
        todo!()
    }
}

/* Deref ******************************************************************** */

impl Deref for Proposals {
    type Target = <Vec<Proposal> as Deref>::Target;
    fn deref(&self) -> &Self::Target {
        self.proposals.deref()
    }
}
