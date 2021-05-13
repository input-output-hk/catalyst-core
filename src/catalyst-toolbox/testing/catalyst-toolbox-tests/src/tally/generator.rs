use chain_impl_mockchain::block::Block;

use chain_addr::{Address, Discrimination, Kind};
use chain_impl_mockchain::certificate::{VotePlan, VotePlanId, VoteTallyPayload};
use chain_impl_mockchain::fee::LinearFee;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::ledger::Ledger;
use chain_impl_mockchain::stake::StakeControl;
use chain_impl_mockchain::testing::create_initial_vote_plan;
use chain_impl_mockchain::transaction::{
    InputEnum, TransactionSlice, UnspecifiedAccountIdentifier,
};
use chain_impl_mockchain::utxo;
use chain_impl_mockchain::vote::{Choice, CommitteeId, VotePlanManager, VotePlanStatus};
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::{
    try_initials_vec_from_messages, Block0Configuration, CommitteeIdDef,
};
use jormungandr_testing_utils::testing::network_builder::{Blockchain, Random, Seed, Settings};
use jormungandr_testing_utils::wallet::Wallet;
use rand::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use std::collections::HashMap;

pub struct VoteRoundGenerator {
    block0: Block0Configuration,
    block0_hash: Hash,
    wallets: HashMap<Address, Wallet>,
    committee_wallets: HashMap<Address, Wallet>,
    voteplan_managers: HashMap<VotePlanId, VotePlanManager>,
}

fn committee_id_to_address(id: CommitteeIdDef) -> Address {
    let id = CommitteeId::from(id);
    let pk = id.public_key();
    chain_addr::Address(Discrimination::Production, Kind::Account(pk))
}

fn account_from_slice<P>(
    transaction_slice: &TransactionSlice<P>,
) -> Option<UnspecifiedAccountIdentifier> {
    let account = transaction_slice.inputs().iter().next().unwrap().to_enum();
    if let InputEnum::AccountInput(account, _) = account {
        Some(account)
    } else {
        None
    }
}

impl VoteRoundGenerator {
    pub fn new<R: Rng>(blockchain: Blockchain, rng: &mut R) -> Self {
        let mut settings = Settings::new(
            HashMap::new(),
            blockchain.clone(),
            &mut Random::new(Seed::generate(rng)),
        );

        let mut wallets = settings
            .wallets
            .values()
            .cloned()
            .map(|w| {
                let wallet = Wallet::from(w);
                (wallet.address().into(), wallet)
            })
            .collect::<HashMap<_, _>>();

        let committee_wallets = settings
            .block0
            .blockchain_configuration
            .committees
            .iter()
            .cloned()
            .map(committee_id_to_address)
            // TODO: if committee members can vote we should not remove them
            .map(|addr| (addr.clone(), wallets.remove(&addr).unwrap()))
            .collect::<HashMap<_, _>>();

        let committee_member: chain_impl_mockchain::testing::data::Wallet =
            committee_wallets.values().next().cloned().unwrap().into();

        let mut vote_plans_fragments = Vec::new();
        let mut voteplan_managers = HashMap::new();
        for vote_plan_def in blockchain.vote_plans() {
            let vote_plan = vote_plan_def.into();
            vote_plans_fragments.push(create_initial_vote_plan(
                &vote_plan,
                &[committee_member.clone()],
            ));
            voteplan_managers.insert(
                vote_plan.to_id(),
                VotePlanManager::new(
                    vote_plan,
                    settings
                        .block0
                        .blockchain_configuration
                        .committees
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect(),
                ),
            );
        }

        settings
            .block0
            .initial
            .extend(try_initials_vec_from_messages(vote_plans_fragments.iter()).unwrap());

        let block0_hash = settings.block0.to_block().header.id().into();
        Self {
            block0: settings.block0,
            block0_hash,
            committee_wallets,
            voteplan_managers,
            wallets,
        }
    }

    pub fn block0(&self) -> Block {
        self.block0.to_block()
    }

    pub fn block0_config(&self) -> &Block0Configuration {
        &self.block0
    }

    pub fn wallets(&mut self) -> &mut HashMap<Address, Wallet> {
        &mut self.wallets
    }

    pub fn voteplans(&self) -> Vec<&VotePlan> {
        self.voteplan_managers
            .values()
            .map(VotePlanManager::plan)
            .collect::<Vec<_>>()
    }

    pub fn generate_vote_fragments<R: Rng>(
        &mut self,
        mut strategy: impl GenerationStrategy,
        n_votes: u32,
        shuffle: bool,
        rng: &mut R,
    ) -> Vec<Fragment> {
        let voteplans = self
            .voteplan_managers
            .values()
            .map(VotePlanManager::plan)
            .collect::<Vec<_>>();
        let mut wallets = self.wallets.values_mut().collect::<Vec<_>>();

        let mut fragments = strategy.generate_fragments(
            &mut wallets,
            &voteplans,
            n_votes,
            &self.block0_hash,
            &self.block0.blockchain_configuration.linear_fees,
        );
        if shuffle {
            fragments.shuffle(rng);
        }

        for fragment in &fragments {
            if let Fragment::VoteCast(ref transaction) = fragment {
                let vote_cast = transaction.as_slice().payload().into_payload();
                let vote_plan_id = vote_cast.vote_plan().clone();

                let address =
                    account_from_slice(&transaction.as_slice()).expect("utxo votes not supported");
                let update_voteplan = self
                    .voteplan_managers
                    .get(&vote_plan_id)
                    .unwrap()
                    .vote(
                        self.voteplan_managers
                            .get(&vote_plan_id)
                            .expect("vote plan not found")
                            .plan()
                            .vote_start(),
                        address,
                        vote_cast,
                    )
                    .unwrap();
                self.voteplan_managers.insert(vote_plan_id, update_voteplan);
            } else {
                panic!("a non vote fragment was generated");
            };
        }

        fragments
    }

    pub fn tally_transactions(&mut self) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        for voteplan in self.voteplan_managers.values() {
            let committee_member = self.committee_wallets.values_mut().next().unwrap();
            let tally_fragment = committee_member
                .issue_vote_tally_cert(
                    &self.block0_hash,
                    &self.block0.blockchain_configuration.linear_fees,
                    voteplan.plan(),
                    VoteTallyPayload::Public,
                )
                .unwrap();
            committee_member.confirm_transaction();
            fragments.push(tally_fragment);
        }
        fragments
    }

    pub fn tally(&mut self) -> Vec<VotePlanStatus> {
        let block0 = self.block0.to_block();
        let tmp_ledger = Ledger::new(block0.header.id(), block0.fragments()).unwrap();
        let stake_control = StakeControl::new_with(tmp_ledger.accounts(), &utxo::Ledger::new());
        let mut res = Vec::new();

        for manager in self.voteplan_managers.values_mut() {
            let vote_end = manager.plan().vote_end();
            *manager = manager
                .public_tally(
                    vote_end,
                    &stake_control.clone(),
                    &Default::default(),
                    self.block0.blockchain_configuration.committees[0].into(),
                    |_| (),
                )
                .unwrap();
            res.push(manager.statuses());
        }

        res
    }
}

pub trait GenerationStrategy {
    fn generate_fragments(
        &mut self,
        wallets: &mut [&mut Wallet],
        voteplans: &[&VotePlan],
        n_fragments: u32,
        block0_hash: &Hash,
        fees: &LinearFee,
    ) -> Vec<Fragment>;
}

pub enum TestStrategy {
    Random([u8; 32]),
}

impl GenerationStrategy for TestStrategy {
    fn generate_fragments(
        &mut self,
        wallets: &mut [&mut Wallet],
        voteplans: &[&VotePlan],
        n_fragments: u32,
        block0_hash: &Hash,
        fees: &LinearFee,
    ) -> Vec<Fragment> {
        match self {
            Self::Random(seed) => {
                let mut rng = ChaChaRng::from_seed(*seed);
                let mut fragments = Vec::new();

                for _ in 0..n_fragments {
                    let wallet = wallets.choose_mut(&mut rng).expect("no wallet");
                    let voteplan = voteplans.choose(&mut rng).expect("no voteplans");
                    let proposal_index = rng.gen_range(0..voteplan.proposals().len());
                    let choice = Choice::new(rng.gen_bool(0.5) as u8 + 1); // app votes ares 1-based

                    let fragment = wallet
                        .issue_vote_cast_cert(
                            block0_hash,
                            fees,
                            voteplan,
                            proposal_index as u8,
                            &choice,
                        )
                        .unwrap();
                    wallet.confirm_transaction();

                    fragments.push(fragment);
                }

                fragments
            }
        }
    }
}
