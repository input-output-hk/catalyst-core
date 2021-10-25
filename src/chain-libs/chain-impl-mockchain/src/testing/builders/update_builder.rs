use chain_crypto::{Ed25519, SecretKey};

use crate::{
    config::ConfigParam,
    fragment::config::ConfigParams,
    key::{signed_new, BftLeaderId},
    update::{
        SignedUpdateProposal, SignedUpdateVote, UpdateProposal, UpdateProposalId,
        UpdateProposalWithProposer, UpdateVote,
    },
};

pub struct ProposalBuilder {
    config_params: ConfigParams,
}

impl Default for ProposalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProposalBuilder {
    pub fn new() -> Self {
        ProposalBuilder {
            config_params: ConfigParams::new(),
        }
    }
    pub fn with_proposal_changes(&mut self, changes: Vec<ConfigParam>) -> &mut Self {
        for change in changes {
            self.with_proposal_change(change);
        }
        self
    }

    pub fn with_proposal_change(&mut self, change: ConfigParam) -> &mut Self {
        self.config_params.push(change);
        self
    }

    pub fn build(&self) -> UpdateProposal {
        UpdateProposal::new(self.config_params.clone())
    }
}

#[derive(Default)]
pub struct SignedProposalBuilder {
    update_proposal: Option<UpdateProposal>,
    proposer_secret_key: Option<SecretKey<Ed25519>>,
}

impl SignedProposalBuilder {
    pub fn new() -> Self {
        SignedProposalBuilder {
            update_proposal: None,
            proposer_secret_key: None,
        }
    }

    pub fn with_proposer_secret_key(
        &mut self,
        proposer_secret_key: SecretKey<Ed25519>,
    ) -> &mut Self {
        self.proposer_secret_key = Some(proposer_secret_key);
        self
    }

    pub fn with_proposal_update(&mut self, update_proposal: UpdateProposal) -> &mut Self {
        self.update_proposal = Some(update_proposal);
        self
    }

    pub fn build(&self) -> SignedUpdateProposal {
        SignedUpdateProposal::new(
            signed_new(
                &self.proposer_secret_key.clone().unwrap(),
                self.update_proposal.clone().unwrap(),
            )
            .sig,
            UpdateProposalWithProposer::new(
                self.update_proposal.clone().unwrap(),
                BftLeaderId(self.proposer_secret_key.clone().unwrap().to_public()),
            ),
        )
    }
}

#[derive(Default)]
pub struct UpdateVoteBuilder {
    proposal_id: Option<UpdateProposalId>,
    voter_secret_key: Option<SecretKey<Ed25519>>,
}

impl UpdateVoteBuilder {
    pub fn new() -> Self {
        UpdateVoteBuilder {
            proposal_id: None,
            voter_secret_key: None,
        }
    }

    pub fn with_proposal_id(&mut self, proposal_id: UpdateProposalId) -> &mut Self {
        self.proposal_id = Some(proposal_id);
        self
    }

    pub fn with_voter_secret_key(&mut self, voter_secret_key: SecretKey<Ed25519>) -> &mut Self {
        self.voter_secret_key = Some(voter_secret_key);
        self
    }

    pub fn build(&self) -> SignedUpdateVote {
        let update_vote = UpdateVote::new(
            self.proposal_id.unwrap(),
            BftLeaderId(self.voter_secret_key.clone().unwrap().to_public()),
        );
        SignedUpdateVote::new(
            signed_new(&self.voter_secret_key.clone().unwrap(), update_vote.clone()).sig,
            update_vote,
        )
    }
}
