use crate::{
    config::ConfigParam,
    fragment::ConfigParams,
    key::{signed_new, BftLeaderId},
    update::{
        SignedUpdateProposal, SignedUpdateVote, UpdateProposal, UpdateProposalId,
        UpdateProposalWithProposer, UpdateVote,
    },
};
use chain_crypto::{Ed25519, SecretKey};

pub fn build_proposal(
    proposer_secret_key: SecretKey<Ed25519>,
    config_params: Vec<ConfigParam>,
) -> SignedUpdateProposal {
    //create proposal
    let update_proposal = UpdateProposal::new(ConfigParams(config_params));

    let proposer_id = BftLeaderId(proposer_secret_key.to_public());

    //add proposer
    let update_proposal_with_proposer =
        UpdateProposalWithProposer::new(update_proposal.clone(), proposer_id);

    //sign proposal
    SignedUpdateProposal::new(
        signed_new(&proposer_secret_key, update_proposal).sig,
        update_proposal_with_proposer,
    )
}

pub fn build_vote(
    proposal_id: UpdateProposalId,
    leader_secret_key: SecretKey<Ed25519>,
) -> SignedUpdateVote {
    let update_vote = UpdateVote::new(proposal_id, BftLeaderId(leader_secret_key.to_public()));
    SignedUpdateVote::new(
        signed_new(&leader_secret_key, update_vote.clone()).sig,
        update_vote,
    )
}
