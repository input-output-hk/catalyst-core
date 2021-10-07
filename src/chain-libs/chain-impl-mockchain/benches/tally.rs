use chain_crypto::testing::TestCryptoRng;
use chain_impl_mockchain::testing::scenario::template::WalletTemplateBuilder;
use chain_impl_mockchain::{
    certificate::{
        DecryptedPrivateTally, DecryptedPrivateTallyProposal, EncryptedVoteTally, VoteCast,
        VotePlan, VoteTally,
    },
    fee::LinearFee,
    header::BlockDate,
    testing::{
        data::CommitteeMembersManager,
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, proposal, vote_plan, wallet},
        VoteTestGen,
    },
    value::Value,
    vote::{Choice, PayloadType},
};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{
    distributions::{Distribution, Uniform, WeightedIndex},
    Rng, SeedableRng,
};
use rayon::prelude::*;

const ALICE: &str = "Alice";
const STAKE_POOL: &str = "stake_pool";
const CRS_SEED: &[u8] = b"This should be a shared seed among the different committee members. Could be the id of the previous VotePlan";
const VOTE_PLAN: &str = "fund1";
const MEMBERS_NO: usize = 3;
const THRESHOLD: usize = 2;

fn tally_benchmark(
    benchmark_name: &str,
    n_proposals: usize,
    voters_count: usize,
    proposals_per_voter_ratio: f64,
    yes_votes_ratio: f64,
    voting_power_distribution: impl Distribution<u64>,
    c: &mut Criterion,
) {
    let mut rng = TestCryptoRng::seed_from_u64(0);

    // All wallets that are needed to be initialized in the genesis block
    // TODO the underlying ledger constructor is not using this &mut. This should be a plain
    // Vec<WalletTemplateBuilder>, which will greatly simplify this code.
    let mut wallets: Vec<&mut WalletTemplateBuilder> = Vec::new();

    // Stake pool owner
    let mut alice_wallet_builder = wallet(ALICE);
    alice_wallet_builder
        .with(1_000)
        .owns(STAKE_POOL)
        .committee_member();
    wallets.push(&mut alice_wallet_builder);

    // generate the required number of wallets from the distribution
    let voters_aliases: Vec<_> = (1..=voters_count)
        .map(|counter| format!("voter_{}", counter))
        .collect();
    let voting_powers: Vec<_> = voting_power_distribution
        .sample_iter(&mut rng)
        .take(voters_count)
        .collect();
    let total_votes = voting_powers.iter().sum();
    let mut voters_wallets: Vec<_> = voters_aliases
        .iter()
        .zip(voting_powers.iter())
        .map(|(alias, voting_power)| {
            let mut wallet_builder = WalletTemplateBuilder::new(alias);
            wallet_builder.with(*voting_power);
            wallet_builder
        })
        .collect();

    wallets.append(&mut voters_wallets.iter_mut().collect());

    // Prepare committee members keys
    let members = CommitteeMembersManager::new(&mut rng, CRS_SEED, THRESHOLD, MEMBERS_NO);
    let committee_keys: Vec<_> = members
        .members()
        .iter()
        .map(|committee_member| committee_member.public_key())
        .collect();

    // Build the vote plan
    let mut vote_plan_builder = vote_plan(VOTE_PLAN);
    vote_plan_builder
        .owner(ALICE)
        .consecutive_epoch_dates()
        .payload_type(PayloadType::Private)
        .committee_keys(committee_keys);
    for _ in 0..n_proposals {
        let mut proposal_builder = proposal(VoteTestGen::external_proposal_id());
        proposal_builder.options(3).action_off_chain();
        vote_plan_builder.with_proposal(&mut proposal_builder);
    }

    // Initialize ledger
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(0, 0, 0))
                .with_rewards(Value(1000)),
        )
        .with_initials(wallets)
        .with_vote_plans(vec![&mut vote_plan_builder])
        .build()
        .unwrap();

    // cast votes
    let vote_plan_def = controller.vote_plan(VOTE_PLAN).unwrap();
    let vote_plan: VotePlan = vote_plan_def.into();

    let mut total_votes_per_proposal = vec![0; n_proposals];
    let mut voters_and_powers: Vec<_> = voters_aliases
        .iter()
        .map(|alias| controller.wallet(alias).unwrap())
        .zip(voting_powers.into_iter())
        .collect();
    let vote_fragments = {
        let mut res = Vec::new();
        for (proposal_idx, proposal) in vote_plan.proposals().iter().enumerate() {
            for (private_voter, voting_power) in voters_and_powers.iter_mut() {
                let should_vote = rng.gen_bool(proposals_per_voter_ratio);
                if !should_vote {
                    continue;
                }

                let choice = Choice::new(rng.gen_bool(yes_votes_ratio) as u8);
                let payload = VoteTestGen::private_vote_cast_payload_for(
                    &vote_plan, proposal, choice, &mut rng,
                );

                res.push(controller.fragment_factory().vote_cast(
                    BlockDate::first(),
                    private_voter,
                    VoteCast::new(vote_plan.to_id(), proposal_idx as u8, payload),
                ));
                total_votes_per_proposal[proposal_idx] += *voting_power;
            }
        }
        res
    };

    // Vote casting is an order of magnitude slower than other operations so
    // let's use just 10 samples
    let mut vote_casting_bench = c.benchmark_group("vote_casting");
    vote_casting_bench.sample_size(10);
    vote_casting_bench.bench_function(&format!("vote_cast_{}", benchmark_name), |b| {
        b.iter(|| {
            let mut ledger = ledger.clone();
            for vote in &vote_fragments {
                ledger.apply_fragment(vote, ledger.date()).unwrap();
            }
        })
    });
    vote_casting_bench.finish();

    // apply votes
    for vote in vote_fragments {
        ledger.apply_fragment(&vote, ledger.date()).unwrap();
    }

    // Proceed to tally
    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    // Get encrypted tally
    let mut alice = controller.wallet(ALICE).unwrap();

    let encrypted_tally = EncryptedVoteTally::new(vote_plan.to_id());
    let fragment = controller.fragment_factory().vote_encrypted_tally(
        BlockDate::first(),
        &alice,
        encrypted_tally,
    );

    let parameters = ledger.parameters.clone();
    let date = ledger.date();

    // benchmark the creation of encrypted tally
    c.bench_function(&format!("vote_encrypted_tally_{}", benchmark_name), |b| {
        b.iter(|| {
            ledger
                .ledger
                .apply_fragment(&parameters, &fragment, date)
                .unwrap();
        })
    });

    // apply encrypted tally fragment
    ledger.apply_fragment(&fragment, ledger.date()).unwrap();
    alice.confirm_transaction();

    // benchmark producing decryption
    let vote_plans = ledger.ledger.active_vote_plans();
    let vote_plan_status = vote_plans
        .iter()
        .find(|c_vote_plan| {
            let vote_plan: VotePlan = vote_plan.clone();
            c_vote_plan.id == vote_plan.to_id()
        })
        .unwrap();
    c.bench_function(&format!("tally_decrypt_share_{}", benchmark_name), |b| {
        b.iter(|| {
            members.members()[0].produce_decrypt_shares(&vote_plan_status);
        })
    });

    // Collect the decryption shares per proposal. Here we get a matrix that
    // we need to transpose.
    let mut decrypt_shares_iter: Vec<_> = members
        .members()
        .iter()
        .map(|member| member.produce_decrypt_shares(&vote_plan_status).into_iter())
        .collect();
    let decrypt_shares: Vec<Vec<_>> = (0..n_proposals)
        .map(|_| {
            decrypt_shares_iter
                .iter_mut()
                .filter_map(|member_shares| member_shares.next())
                .collect()
        })
        .collect();

    let decrypt_tally = || {
        let table = chain_vote::TallyOptimizationTable::generate(total_votes);

        vote_plan_status
            .proposals
            .par_iter()
            .enumerate()
            .map(|(i, proposal)| {
                proposal
                    .tally
                    .clone()
                    .unwrap()
                    .private_encrypted()
                    .unwrap()
                    .0
                    .validate_partial_decryptions(
                        &vote_plan.committee_public_keys(),
                        &decrypt_shares[i],
                    )
                    .unwrap()
                    .decrypt_tally(total_votes_per_proposal[i], &table)
                    .unwrap()
            })
            .collect::<Vec<_>>()
    };

    c.bench_function(&format!("decrypt_private_tally_{}", benchmark_name), |b| {
        b.iter(decrypt_tally)
    });

    let shares = decrypt_tally()
        .into_iter()
        .zip(decrypt_shares.into_iter())
        .map(|(tally, decrypt_shares)| DecryptedPrivateTallyProposal {
            decrypt_shares: decrypt_shares.into_boxed_slice(),
            tally_result: tally.votes.into_boxed_slice(),
        })
        .collect();

    let decrypted_tally = VoteTally::new_private(
        vote_plan.to_id(),
        DecryptedPrivateTally::new(shares).unwrap(),
    );
    let fragment =
        controller
            .fragment_factory()
            .vote_tally(BlockDate::first(), &alice, decrypted_tally);

    c.bench_function(&format!("vote_tally_{}", benchmark_name), |b| {
        b.iter(|| {
            ledger
                .ledger
                .apply_fragment(&parameters, &fragment, ledger.date())
                .unwrap();
        })
    });

    ledger.apply_fragment(&fragment, ledger.date()).unwrap();
}

fn tally_benchmark_flat_distribution(
    benchmark_name: &str,
    voters_count: usize,
    voting_power_per_voter: u64,
    c: &mut Criterion,
) {
    let voting_power_distribution = rand::distributions::uniform::Uniform::from(
        voting_power_per_voter..voting_power_per_voter + 1,
    );
    tally_benchmark(
        benchmark_name,
        1,
        voters_count,
        1.0,
        0.5,
        voting_power_distribution,
        c,
    );
}

fn tally_benchmark_128_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution("128_voters_1000_ada", 128, 1000, c);
}

fn tally_benchmark_200_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution("200_voters_1000_ada", 200, 1000, c);
}

fn tally_benchmark_200_voters_1_000_000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution("200_voters_1_000_000_ada", 200, 1_000_000, c);
}

fn tally_benchmark_1000_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution("1000_voters_1000_ada", 1000, 1000, c);
}

struct FundDistribution {
    ranges_no_sampler: WeightedIndex<f64>,
    ranges_values_samplers: Vec<Uniform<u64>>,
}

impl FundDistribution {
    pub fn new(ranges_bounds: &[u64], ranges_weights: &[f64]) -> Self {
        assert_eq!(ranges_bounds.len(), ranges_weights.len() + 1);
        let ranges_no_sampler = WeightedIndex::new(ranges_weights).unwrap();
        let ranges_values_samplers = ranges_bounds
            .windows(2)
            .map(|bounds| Uniform::from(bounds[0]..bounds[1]))
            .collect();
        Self {
            ranges_no_sampler,
            ranges_values_samplers,
        }
    }
}

impl Distribution<u64> for FundDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u64 {
        let range_no = self.ranges_no_sampler.sample(rng);
        self.ranges_values_samplers[range_no].sample(rng)
    }
}

fn tally_benchmark_fund3_scenario(c: &mut Criterion) {
    // 15k voters
    // 150 proposals
    // Each voter to vote on 75% of proposals
    // Distribution: 20% users over 1 million, 40% between 1 million to 200k, 40% below 200k
    // 65% no, 35% yes. 0% abstain
    // Threshold: 3000
    let voters_count = 15_000;
    let n_proposals = 150;
    let proposals_per_voter_ratio = 0.75;
    let ranges_bounds = &[3_000, 200_000, 1_000_000, 10_000_000];
    let ranges_weights = &[0.4, 0.4, 0.2];
    let yes_votes_ratio = 0.35;

    let voting_power_distribution = FundDistribution::new(ranges_bounds, ranges_weights);

    tally_benchmark(
        "fund3_scenario",
        n_proposals,
        voters_count,
        proposals_per_voter_ratio,
        yes_votes_ratio,
        voting_power_distribution,
        c,
    );
}

fn tally_benchmark_fund4_scenario(c: &mut Criterion) {
    // 30k voters
    // 300 proposals
    // Each voter to vote on 75% of proposals
    // Distribution: 20% users over 1 million, 40% between 1 million to 200k, 40% below 200k
    // 65% no, 35% yes. 0% abstain
    // Threshold: 3000
    let voters_count = 30_000;
    let n_proposals = 300;
    let proposals_per_voter_ratio = 0.75;
    let ranges_bounds = &[3_000, 200_000, 1_000_000, 10_000_000];
    let ranges_weights = &[0.4, 0.4, 0.2];
    let yes_votes_ratio = 0.35;

    let voting_power_distribution = FundDistribution::new(ranges_bounds, ranges_weights);

    tally_benchmark(
        "fund4_scenario",
        n_proposals,
        voters_count,
        proposals_per_voter_ratio,
        yes_votes_ratio,
        voting_power_distribution,
        c,
    );
}

criterion_group!(
    fast_bench,
    tally_benchmark_128_voters_1000_ada,
    tally_benchmark_200_voters_1000_ada,
    tally_benchmark_200_voters_1_000_000_ada,
    tally_benchmark_1000_voters_1000_ada,
);
criterion_group!(
    big_bench,
    tally_benchmark_fund3_scenario,
    tally_benchmark_fund4_scenario,
);
criterion_main!(fast_bench, big_bench);
