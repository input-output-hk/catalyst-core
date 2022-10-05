use chain_addr::{Address, Discrimination};
use chain_impl_mockchain::{
    certificate::VotePlan,
    chaintypes::ConsensusVersion,
    fee::LinearFee,
    testing::{
        create_initial_vote_plan,
        data::CommitteeMembersManager,
        scenario::{proposal, template::VotePlanDefBuilder, vote_plan},
        VoteTestGen,
    },
    tokens::{
        identifier::TokenIdentifier,
        minting_policy::MintingPolicy,
        name::{TokenName, TOKEN_NAME_MAX_SIZE},
        policy_hash::{PolicyHash, POLICY_HASH_SIZE},
    },
    vote::PayloadType,
};
use jormungandr_lib::interfaces::{
    try_initial_fragment_from_message, Block0Configuration, BlockchainConfiguration, Initial,
    InitialToken,
};

use rand::{CryptoRng, Rng};
use std::{collections::HashMap, convert::TryFrom};
use thor::Wallet;

// rather arbitrary at this point
const DEFAULT_WALLETS: u32 = 10;
const DEFAULT_COMMITTEES: u8 = 2;
const DEFAULT_PRIVATE_COMMITTEE_SIZE: usize = 1;
const MIN_FUNDS: u64 = 500;
const MAX_FUNDS: u64 = 50000;

pub struct TestBlockchainBuilder {
    n_wallets: u32,
    n_committees: u8,
    vote_plans: Vec<VotePlanDefBuilder>,
}

pub struct TestBlockchain {
    pub config: Block0Configuration,
    pub wallets: HashMap<Address, Wallet>,
    pub committee_wallets: HashMap<Address, Wallet>,
    pub committee_manager: CommitteeMembersManager,
    pub vote_plans: Vec<VotePlan>,
    pub voting_token: TokenIdentifier,
}

impl TestBlockchainBuilder {
    pub fn new() -> Self {
        TestBlockchainBuilder {
            n_wallets: DEFAULT_WALLETS,
            n_committees: DEFAULT_COMMITTEES,
            vote_plans: Vec::new(),
        }
    }

    pub fn with_n_wallets(self, n_wallets: u32) -> Self {
        Self { n_wallets, ..self }
    }

    #[allow(dead_code)]
    pub fn with_n_committess(self, n_committees: u8) -> Self {
        Self {
            n_committees,
            ..self
        }
    }

    pub fn with_voteplan(
        self,
        start_epoch: u32,
        tally_epoch: u32,
        end_epoch: u32,
        n_proposals: u8,
        payload_type: PayloadType,
    ) -> Self {
        let mut vote_plans = self.vote_plans;
        let mut vote_plan_builder = vote_plan("ignored");
        vote_plan_builder
            .owner("ignored")
            .vote_phases(start_epoch, tally_epoch, end_epoch)
            .payload_type(payload_type);

        for _ in 0..n_proposals {
            let mut proposal_builder = proposal(VoteTestGen::external_proposal_id());
            proposal_builder.options(3).action_off_chain();
            vote_plan_builder.with_proposal(&mut proposal_builder);
        }

        vote_plans.push(vote_plan_builder);

        Self { vote_plans, ..self }
    }

    pub fn build<R: Rng + CryptoRng>(self, rng: &mut R) -> TestBlockchain {
        let voting_token = TokenIdentifier {
            policy_hash: PolicyHash::from([0u8; POLICY_HASH_SIZE]),
            token_name: TokenName::try_from(vec![0u8; TOKEN_NAME_MAX_SIZE]).unwrap(),
        };

        let mut initial = Vec::with_capacity(self.n_wallets as usize + self.n_committees as usize);
        let mut wallets = (0..self.n_wallets + self.n_committees as u32)
            .into_iter()
            .map(|_| {
                let funds = rng.gen_range(MIN_FUNDS..MAX_FUNDS);
                let wallet =
                    Wallet::new_account_with_discrimination(rng, Discrimination::Production);
                initial.push(Initial::Fund(vec![wallet.to_initial_fund(funds)]));

                // FIXME: this works because it's the default token in the vote plan builder, but it
                // may be better to extract this out and set it explicitly.
                initial.push(Initial::Token(InitialToken {
                    token_id: voting_token.clone().into(),
                    policy: MintingPolicy::new().into(),
                    to: vec![wallet.to_initial_token(funds)],
                }));

                (wallet.address().into(), wallet)
            })
            .collect::<Vec<_>>();

        let committee_wallets = wallets.split_off(self.n_wallets as usize);

        let committee_manager = CommitteeMembersManager::new(
            rng,
            "random crs hash".as_bytes(),
            DEFAULT_PRIVATE_COMMITTEE_SIZE,
            DEFAULT_PRIVATE_COMMITTEE_SIZE,
        );
        let committee_keys = committee_manager
            .members()
            .iter()
            .map(|member| member.public_key())
            .collect::<Vec<_>>();

        let mut vote_plans_fragments = Vec::with_capacity(self.vote_plans.len());
        let vote_plans = self
            .vote_plans
            .into_iter()
            .map(|mut vote_plan| {
                vote_plan.committee_keys(committee_keys.clone());
                let vote_plan = vote_plan.build();
                vote_plans_fragments.push(create_initial_vote_plan(
                    &vote_plan.clone().into(),
                    &[committee_wallets[0].1.clone().into()],
                ));
                vote_plan.into()
            })
            .collect::<Vec<VotePlan>>();

        let discrimination = Discrimination::Production;
        initial.extend(
            vote_plans_fragments
                .iter()
                .map(|message| try_initial_fragment_from_message(discrimination, message).unwrap()),
        );

        let mut config = Block0Configuration {
            blockchain_configuration: BlockchainConfiguration::new(
                discrimination,
                ConsensusVersion::Bft,
                LinearFee::new(0, 0, 0), // it is much easier not to account for feers in the tests verification alg
            ),
            initial,
        };

        config.blockchain_configuration.committees.extend(
            committee_wallets
                .iter()
                .map(|(_addr, wlt): &(Address, Wallet)| wlt.to_committee_id()),
        );
        config
            .blockchain_configuration
            .consensus_leader_ids
            .push(committee_wallets[0].1.identifier().into());

        TestBlockchain {
            config,
            committee_manager,
            wallets: wallets.into_iter().collect::<HashMap<_, _>>(),
            committee_wallets: committee_wallets.into_iter().collect::<HashMap<_, _>>(),
            vote_plans,
            voting_token,
        }
    }
}
