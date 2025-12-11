use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::{certificate::VotePlanId, vote::Options};
use chain_vote::ElectionPublicKey;
use std::str;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
pub use vit_servicing_station_lib::{
    db::models::challenges::Challenge, db::models::community_advisors_reviews::AdvisorReview,
    db::models::funds::Fund, db::models::proposals::Proposal,
    v0::endpoints::service_version::ServiceVersion as VitVersion,
};

pub trait ProposalExtension {
    fn chain_proposal_id_as_str(&self) -> String;
    fn into_wallet_proposal(self) -> wallet_core::Proposal;
}

impl ProposalExtension for FullProposalInfo {
    fn chain_proposal_id_as_str(&self) -> String {
        str::from_utf8(&self.proposal.chain_proposal_id)
            .unwrap()
            .to_string()
    }

    fn into_wallet_proposal(self) -> wallet_core::Proposal {
        let chain_proposal_index = self.voteplan.chain_proposal_index as u8;
        let bytes = hex::decode(self.voteplan.chain_voteplan_id).unwrap();
        let mut vote_plan_id = [0; 32];
        let bytes = &bytes[..vote_plan_id.len()]; // panics if not enough data
        vote_plan_id.copy_from_slice(bytes);

        if self.proposal.chain_voteplan_payload == "public" {
            return wallet_core::Proposal::new_public(
                VotePlanId::from(vote_plan_id),
                chain_proposal_index,
                Options::new_length(self.proposal.chain_vote_options.0.len() as u8).unwrap(),
            );
        }
        wallet_core::Proposal::new_private(
            VotePlanId::from(vote_plan_id),
            chain_proposal_index,
            Options::new_length(self.proposal.chain_vote_options.0.len() as u8).unwrap(),
            ElectionPublicKey::try_from_bech32_str(&self.proposal.chain_vote_encryption_key)
                .unwrap(),
        )
    }
}
