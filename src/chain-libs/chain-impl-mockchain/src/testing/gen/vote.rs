use super::TestGen;
use crate::{
    block::BlockDate,
    certificate::{
        ExternalProposalId, Proposal, Proposals, PushProposal, VoteCastPayload, VoteOptions,
        VotePlan, VoteCast
    },
};
use chain_core::property::BlockDate as BlockDateProp;
use chain_crypto::digest::DigestOf;
use typed_bytes::ByteBuilder;

pub struct VoteTestGen;

impl VoteTestGen {
    pub fn proposal() -> Proposal {
        Proposal::new(
            VoteTestGen::external_proposal_id(),
            VoteOptions::new_length(4),
        )
    }

    pub fn proposals(count: usize) -> Proposals {
        let mut proposals = Proposals::new();
        for _ in 0..count {
            assert_eq!(
                PushProposal::Success,
                proposals.push(VoteTestGen::proposal()),
                "generate_proposal method is only for correct data preparation"
            );
        }
        proposals
    }

    pub fn external_proposal_id() -> ExternalProposalId {
        DigestOf::digest_byteslice(
            &ByteBuilder::new()
                .bytes(&TestGen::bytes())
                .finalize()
                .as_byteslice(),
        )
    }

    pub fn vote_plan() -> VotePlan {
        VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            VoteTestGen::proposals(3),
        )
    }

    pub fn vote_plan_with_proposals(count: usize) -> VotePlan {
        VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            VoteTestGen::proposals(count),
        )
    }

    pub fn vote_cast_payload() -> VoteCastPayload {
        VoteCastPayload::new(TestGen::bytes().to_vec())
    }

    pub fn vote_cast_for(vote_plan: &VotePlan) -> VoteCast {
        VoteCast::new(vote_plan.to_id(), 0, VoteTestGen::vote_cast_payload())
    }
}
