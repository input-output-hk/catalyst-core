use crate::{
    certificate::{ExternalProposalId, VotePlanId},
    date::BlockDate,
    vote::{Options, PayloadType, Tally},
};

pub struct VotePlanStatus {
    pub id: VotePlanId,
    pub payload: PayloadType,
    pub vote_start: BlockDate,
    pub vote_end: BlockDate,
    pub committee_end: BlockDate,
    pub proposals: Vec<VoteProposalStatus>,
}

pub struct VoteProposalStatus {
    pub index: u8,
    pub proposal_id: ExternalProposalId,
    pub options: Options,
    pub tally: Option<Tally>,
}
