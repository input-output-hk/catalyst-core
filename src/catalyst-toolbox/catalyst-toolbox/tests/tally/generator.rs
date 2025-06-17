use chain_impl_mockchain::block::Block;

use super::blockchain::TestBlockchain;
use chain_addr::Address;
use chain_impl_mockchain::ledger::token_distribution::TokenDistribution;
use chain_impl_mockchain::{
    certificate::{
        DecryptedPrivateTally, DecryptedPrivateTallyProposal, VotePlan, VotePlanId,
        VoteTallyPayload,
    },
    fee::LinearFee,
    fragment::Fragment,
    ledger::Ledger,
    stake::StakeControl,
    testing::data::CommitteeMembersManager,
    transaction::{InputEnum, TransactionSlice, UnspecifiedAccountIdentifier},
    utxo,
    vote::{Choice, PayloadType, VotePlanManager, VotePlanStatus},
};
use jormungandr_lib::{crypto::hash::Hash, interfaces::Block0Configuration};
use rand::prelude::*;
use rand_chacha::ChaChaRng;
use std::collections::HashMap;
use std::num::NonZeroU64;
use thor::FragmentBuilder;
use thor::Wallet;

pub struct VoteRoundGenerator {
    block0: Block0Configuration,
    block0_hash: Hash,
    wallets: HashMap<Address, Wallet>,
    committee_wallets: HashMap<Address, Wallet>,
    voteplan_managers: HashMap<VotePlanId, VotePlanManager>,
    committee_manager: CommitteeMembersManager,
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
    pub fn new(blockchain: TestBlockchain) -> Self {
        let TestBlockchain {
            config,
            wallets,
            committee_wallets,
            committee_manager,
            vote_plans,
            ..
        } = blockchain;

        let mut voteplan_managers = HashMap::new();
        for vote_plan in vote_plans {
            voteplan_managers.insert(
                vote_plan.to_id(),
                VotePlanManager::new(
                    vote_plan,
                    config
                        .blockchain_configuration
                        .committees
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect(),
                ),
            );
        }

        let block0_hash = config.to_block().header().id().into();
        Self {
            block0: config,
            block0_hash,
            committee_wallets,
            committee_manager,
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

    pub fn committee_wallets(&mut self) -> &mut HashMap<Address, Wallet> {
        &mut self.committee_wallets
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

        let ledger = self.ledger();
        for fragment in &fragments {
            self.feed_vote_cast(fragment, ledger.token_distribution());
        }

        fragments
    }

    /// Since this has no internal ledger, it assumes all transactions are correctly signed and
    /// within their validity window
    pub fn feed_vote_cast(
        &mut self,
        fragment: &Fragment,
        token_distribution: TokenDistribution<()>,
    ) {
        if let Fragment::VoteCast(ref transaction) = fragment {
            let vote_cast = transaction.as_slice().payload().into_payload();
            let vote_plan_id = vote_cast.vote_plan().clone();
            let address =
                account_from_slice(&transaction.as_slice()).expect("utxo votes not supported");
            let update_voteplan = self.voteplan_managers.get(&vote_plan_id).unwrap().vote(
                self.voteplan_managers
                    .get(&vote_plan_id)
                    .expect("vote plan not found")
                    .plan()
                    .vote_start(),
                address.to_single_account().unwrap(),
                vote_cast,
                token_distribution,
            );
            if let Ok(update_voteplan) = update_voteplan {
                self.voteplan_managers.insert(vote_plan_id, update_voteplan);
            }
        } else {
            panic!("a non vote fragment was generated");
        };
    }

    pub fn ledger(&self) -> Ledger {
        let block0 = self.block0.to_block();
        Ledger::new(block0.header().id(), block0.fragments()).unwrap()
    }

    /// Tally voteplans and return the fragments to run the tally in a separate ledger
    pub fn tally_transactions<R: Rng + CryptoRng>(&mut self, rng: &mut R) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        let member_keys = self
            .committee_manager
            .members()
            .iter()
            .map(|member| member.public_key())
            .collect::<Vec<_>>();

        let tmp_ledger = self.ledger();
        let stake_control = StakeControl::new_with(tmp_ledger.accounts(), &utxo::Ledger::new());
        let table = chain_vote::TallyOptimizationTable::generate(
            NonZeroU64::new(stake_control.assigned().into()).unwrap(),
        );

        self.voteplan_managers = self
            .voteplan_managers
            .clone() // cloning is cheap and make the borrowck happy
            .into_iter()
            .map(|(id, manager)| {
                let vote_end = manager.plan().vote_end();

                match manager.plan().payload_type() {
                    PayloadType::Private => {
                        let mut results = Vec::new();
                        let mut shares = Vec::new();
                        for proposal in manager.statuses().proposals {
                            let encrypted_tally = proposal.tally.private_encrypted().unwrap();

                            let sh = self
                                .committee_manager
                                .members()
                                .iter()
                                .map(|member| {
                                    encrypted_tally.partial_decrypt(rng, member.secret_key())
                                })
                                .collect::<Box<[_]>>();
                            let partial_res = encrypted_tally
                                .validate_partial_decryptions(&member_keys, &sh)
                                .unwrap()
                                .decrypt_tally(&table)
                                .unwrap();
                            results.push(partial_res.votes.into_boxed_slice());
                            shares.push(sh);
                        }

                        let decrypted_tally = DecryptedPrivateTally::new(
                            results
                                .into_iter()
                                .zip(shares)
                                .map(|(tally_result, decrypt_shares)| {
                                    DecryptedPrivateTallyProposal {
                                        decrypt_shares,
                                        tally_result,
                                    }
                                })
                                .collect::<Vec<_>>(),
                        )
                        .unwrap();

                        let manager = manager
                            .private_tally(
                                vote_end,
                                &decrypted_tally,
                                &Default::default(),
                                self.block0.blockchain_configuration.committees[0].into(),
                                tmp_ledger.token_distribution(),
                                |_| (),
                            )
                            .unwrap();

                        fragments.extend(self.prepare_tally_fragments(
                            manager.plan(),
                            VoteTallyPayload::Private {
                                inner: decrypted_tally,
                            },
                        ));

                        (id, manager)
                    }
                    PayloadType::Public => {
                        let manager = manager
                            .public_tally(
                                vote_end,
                                &Default::default(),
                                self.block0.blockchain_configuration.committees[0].into(),
                                tmp_ledger.token_distribution(),
                                |_| (),
                            )
                            .unwrap();

                        fragments.extend(
                            self.prepare_tally_fragments(manager.plan(), VoteTallyPayload::Public),
                        );

                        (id, manager)
                    }
                }
            })
            .collect::<HashMap<_, _>>();
        fragments
    }

    fn prepare_tally_fragments(
        &mut self,
        voteplan: &VotePlan,
        payload: VoteTallyPayload,
    ) -> Vec<Fragment> {
        let mut res = Vec::new();
        let committee_end = voteplan.committee_end();
        let committee_member = self.committee_wallets.values_mut().next().unwrap();

        let fragment_builder = FragmentBuilder::new(
            &self.block0_hash,
            &self.block0.blockchain_configuration.linear_fees,
            committee_end,
        );

        let tally_fragment = fragment_builder.vote_tally(committee_member, voteplan, payload);
        committee_member.confirm_transaction();
        res.push(tally_fragment);

        res
    }

    pub fn statuses(&self) -> Vec<VotePlanStatus> {
        self.voteplan_managers
            .values()
            .map(|manager| manager.statuses())
            .collect::<Vec<_>>()
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
                    let valid_until = voteplan.vote_start().next_epoch();
                    let fragment = FragmentBuilder::new(block0_hash, fees, valid_until).vote_cast(
                        wallet,
                        voteplan,
                        proposal_index as u8,
                        &choice,
                    );
                    wallet.confirm_transaction();

                    fragments.push(fragment);
                }

                fragments
            }
        }
    }
}
