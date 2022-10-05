use crate::{
    certificate::{UpdateProposal, UpdateProposalId, UpdateVote},
    config::ConfigParam,
    fee::LinearFee,
    fragment::config::ConfigParams,
    key::BftLeaderId,
    testing::arbitrary::utils as arbitrary_utils,
    testing::data::LeaderPair,
};
use chain_crypto::{Ed25519, SecretKey};
use quickcheck::{Arbitrary, Gen};
use std::fmt::Debug;
use std::{collections::HashMap, iter};

#[derive(Clone, Debug)]
pub struct UpdateProposalData {
    pub leaders: HashMap<BftLeaderId, SecretKey<Ed25519>>,
    pub voters: HashMap<BftLeaderId, SecretKey<Ed25519>>,
    pub proposal: UpdateProposal,
    pub block_signing_key: SecretKey<Ed25519>,
}

impl UpdateProposalData {
    pub fn leaders_ids(&self) -> Vec<BftLeaderId> {
        self.leaders.keys().cloned().collect()
    }

    pub fn leaders_pairs(&self) -> Vec<LeaderPair> {
        self.leaders
            .values()
            .cloned()
            .map(LeaderPair::new)
            .collect()
    }

    pub fn proposal_settings(&self) -> ConfigParams {
        self.proposal.changes().clone()
    }

    pub fn gen_votes(&self, proposal_id: UpdateProposalId) -> Vec<UpdateVote> {
        self.voters
            .iter()
            .map(|(id, _)| UpdateVote::new(proposal_id, id.clone()))
            .collect()
    }
}

impl Arbitrary for UpdateProposalData {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        let leader_size = 1; //usize::arbitrary(gen) % 20 + 1;
        let leaders: HashMap<BftLeaderId, SecretKey<Ed25519>> = iter::from_fn(|| {
            let sk: SecretKey<Ed25519> = Arbitrary::arbitrary(gen);
            let leader_id = BftLeaderId(sk.to_public());
            Some((leader_id, sk))
        })
        .take(leader_size)
        .collect();

        let voters: HashMap<BftLeaderId, SecretKey<Ed25519>> =
            arbitrary_utils::choose_random_map_subset(&leaders, gen);

        //create proposal
        let unique_arbitrary_settings: Vec<ConfigParam> = vec![
            ConfigParam::SlotsPerEpoch(u32::arbitrary(gen)),
            ConfigParam::SlotDuration(u8::arbitrary(gen)),
            ConfigParam::EpochStabilityDepth(u32::arbitrary(gen)),
            ConfigParam::BlockContentMaxSize(u32::arbitrary(gen)),
            ConfigParam::LinearFee(LinearFee::arbitrary(gen)),
            ConfigParam::ProposalExpiration(u32::arbitrary(gen)),
        ];

        let proposal = UpdateProposal::new(
            ConfigParams(arbitrary_utils::choose_random_vec_subset(
                &unique_arbitrary_settings,
                gen,
            )),
            leaders.iter().next().unwrap().0.clone(),
        );

        let sk: chain_crypto::SecretKey<Ed25519> = Arbitrary::arbitrary(gen);

        UpdateProposalData {
            leaders: leaders.into_iter().collect(),
            voters: voters.into_iter().collect(),
            proposal,
            block_signing_key: sk,
        }
    }
}
