pub use chain_impl_mockchain::chaintypes::ConsensusVersion;
use chain_impl_mockchain::{
    fee::LinearFee,
    testing::{
        scenario::{proposal, template::VotePlanDef, vote_plan},
        VoteTestGen,
    },
};
use jormungandr_lib::interfaces::{
    ActiveSlotCoefficient, KesUpdateSpeed, NumberOfSlotsPerEpoch, SlotDuration,
};
use jormungandr_testing_utils::testing::network_builder::{Blockchain, WalletTemplate};
use rand::Rng;

// rather arbitrary at this point
const DEFAULT_WALLETS: u32 = 10;
const DEFAULT_COMMITTEES: u8 = 2;
const MIN_FUNDS: u64 = 500;
const MAX_FUNDS: u64 = 50000;

pub struct BlockchainBuilder {
    n_wallets: u32,
    n_committees: u8,
    vote_plans: Vec<VotePlanDef>,
}

impl BlockchainBuilder {
    pub fn new() -> Self {
        BlockchainBuilder {
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

    pub fn with_public_voteplan(
        self,
        start_epoch: u32,
        tally_epoch: u32,
        end_epoch: u32,
        n_proposals: u8,
    ) -> Self {
        let mut vote_plans = self.vote_plans;
        let mut vote_plan_builder = vote_plan("ignored");
        vote_plan_builder
            .owner("ignored")
            .vote_phases(start_epoch, tally_epoch, end_epoch);

        for _ in 0..n_proposals {
            let mut proposal_builder = proposal(VoteTestGen::external_proposal_id());
            proposal_builder.options(3).action_off_chain();
            vote_plan_builder.with_proposal(&mut proposal_builder);
        }

        vote_plans.push(vote_plan_builder.build());

        Self { vote_plans, ..self }
    }

    pub fn build<R: Rng>(mut self, rng: &mut R) -> Blockchain {
        let mut blockchain = Blockchain::new(
            ConsensusVersion::Bft,
            NumberOfSlotsPerEpoch::default(),
            SlotDuration::default(),
            KesUpdateSpeed::default(),
            ActiveSlotCoefficient::default(),
        );
        blockchain.add_leader("we just need a consensus leader id");
        blockchain.set_discrimination(chain_addr::Discrimination::Production);
        // it is much easier not to account for feers in the tests verification alg
        blockchain.set_linear_fee(LinearFee::new(0, 0, 0));
        for _ in 0..self.n_wallets {
            let name: String = rng
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();
            let funds = rng.gen_range(MIN_FUNDS..MAX_FUNDS);
            let wallet = WalletTemplate::new_account(
                &name,
                chain_impl_mockchain::value::Value(funds),
                blockchain.discrimination(),
            );

            blockchain.add_wallet(wallet);
            if self.n_committees > 0 {
                blockchain.add_committee(name);
                self.n_committees -= 1;
            }
        }
        for vote_plan in self.vote_plans {
            blockchain.add_vote_plan(vote_plan);
        }

        blockchain
    }
}
