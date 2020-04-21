use crate::{
    certificate::CertificateSlice,
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
    value::Value,
};
use chain_addr::Address;
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use chain_crypto::{digest::DigestOf, Blake2b256};
use chain_time::{DurationSeconds, TimeOffsetSeconds};
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    funding_plan: FundingPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FundingPlan {
    /// may the associated proposal be voted for, the funding
    /// will be paid upfront to the given address.
    UpFront {
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

    fn serialize_in(&self, bb: ByteBuilder<VotePlan>) -> ByteBuilder<VotePlan> {
        let bb = bb
            .bytes(self.external_id.as_ref())
            .sub(|sbb| self.funding_plan.serialize_in(sbb));
        bb
    }
}

impl FundingPlan {
    fn serialize_in(&self, bb: ByteBuilder<VotePlan>) -> ByteBuilder<VotePlan> {
        match self {
            Self::UpFront { value, address } => bb.u8(0).bytes(&address.to_bytes()).u64(value.0),
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
}

impl VotePlan {
    pub fn new(
        vote_start: TimeOffsetSeconds,
        vote_duration: DurationSeconds,
        committee_duration: DurationSeconds,
        proposals: Proposals,
    ) -> Self {
        Self {
            vote_start,
            vote_duration,
            committee_duration,
            proposals,
        }
    }

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

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        let bb = bb
            .u64(self.vote_start.into())
            .u64(self.vote_duration.into())
            .u64(self.committee_duration.into())
            .iter8(&mut self.proposals.iter(), |bb, proposal| {
                proposal.serialize_in(bb)
            });
        bb
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
    const HAS_DATA: bool = false;
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
        todo!()
    }

    fn to_certificate_slice<'a>(p: PayloadSlice<'a, Self>) -> Option<CertificateSlice<'a>> {
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
        let vote_start = DurationSeconds(buf.get_u64()?).into();
        let vote_duration = DurationSeconds(buf.get_u64()?);
        let committee_duration = DurationSeconds(buf.get_u64()?);

        let proposal_size = buf.get_u8()? as usize;
        let mut proposals = Proposals {
            proposals: Vec::with_capacity(proposal_size),
        };
        for i in 0..proposal_size {
            let external_id = <[u8; 32]>::read(buf)?.into();

            let funding_plan_type = buf.get_u8()?;
            let funding_plan = match funding_plan_type {
                0 => {
                    let address = Address::read(buf)?;
                    let value = Value::read(buf)?;
                    FundingPlan::UpFront { address, value }
                }
                _ => {
                    return Err(ReadError::StructureInvalid(format!(
                        "Proposal {index} does not have a known funding plan type: {t}",
                        t = funding_plan_type,
                        index = i,
                    )))
                }
            };

            let proposal = Proposal {
                external_id,
                funding_plan,
            };

            proposals.proposals.push(proposal);
        }

        Ok(Self {
            vote_start,
            vote_duration,
            committee_duration,
            proposals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn serialize_deserialize(vote_plan: VotePlan) -> bool {
        let serialized = vote_plan.serialize();

        let mut buf = ReadBuf::from(serialized.as_ref());
        let result = VotePlan::read(&mut buf);

        let decoded = result.expect("can decode encoded vote plan");

        decoded == vote_plan
    }
}
