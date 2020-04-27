use crate::{
    config::ConfigParam,
    key::BftLeaderId,
    update::{
        SignedUpdateProposal, SignedUpdateVote, UpdateProposal, UpdateProposalId,
        UpdateProposalWithProposer, UpdateVote,
    },
};

pub fn build_proposal(
    proposer_id: BftLeaderId,
    config_params: Vec<ConfigParam>,
) -> SignedUpdateProposal {
    //create proposal
    let mut update_proposal = UpdateProposal::new();

    for config_param in config_params {
        update_proposal.changes.push(config_param);
    }

    //add proposer
    let update_proposal_with_proposer = UpdateProposalWithProposer {
        proposal: update_proposal,
        proposer_id,
    };

    //sign proposal
    SignedUpdateProposal {
        proposal: update_proposal_with_proposer,
    }
}

pub fn build_vote(proposal_id: UpdateProposalId, leader_id: BftLeaderId) -> SignedUpdateVote {
    let update_vote = UpdateVote {
        proposal_id,
        voter_id: leader_id,
    };
    SignedUpdateVote { vote: update_vote }
}
