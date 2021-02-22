use chain_crypto::testing::TestCryptoRng;
use chain_impl_mockchain::testing::scenario::template::WalletTemplateBuilder;
use chain_impl_mockchain::{
    certificate::{
        DecryptedPrivateTally, DecryptedPrivateTallyProposal, EncryptedVoteTally, VotePlan,
        VoteTally,
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
    distributions::{Distribution, WeightedIndex},
    SeedableRng,
};

const ALICE: &str = "Alice";
const STAKE_POOL: &str = "stake_pool";
const VOTE_PLAN: &str = "fund1";
const MEMBERS_NO: usize = 3;
const THRESHOLD: usize = 2;

fn tally_benchmark(
    n_proposals: usize,
    voters_count: usize,
    proposals_per_voter_ratio: f64,
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
    let members = CommitteeMembersManager::new(&mut rng, THRESHOLD, MEMBERS_NO);
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
        proposal_builder.options(3).action_parameters_no_op();
        vote_plan_builder.with_proposal(&mut proposal_builder);
    }

    // Initialize ledger
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_fee(LinearFee::new(0, 0, 0))
                .with_rewards(Value(1000)),
        )
        .with_initials(wallets)
        .with_vote_plans(vec![&mut vote_plan_builder])
        .build()
        .unwrap();

    // cast votes
    let favorable = Choice::new(1);

    let vote_plan_def = controller.vote_plan(VOTE_PLAN).unwrap();
    let vote_plan: VotePlan = vote_plan_def.clone().into();

    let should_vote_distribution =
        WeightedIndex::new(&[1.0 - proposals_per_voter_ratio, proposals_per_voter_ratio]).unwrap();

    let mut total_votes_per_proposal = vec![0; n_proposals];

    for (i, proposal) in vote_plan.proposals().iter().enumerate() {
        for (alias, voting_power) in voters_aliases.iter().zip(voting_powers.iter()) {
            let should_vote = should_vote_distribution.sample(&mut rng);
            if should_vote == 0 {
                continue;
            }

            let mut private_voter = controller.wallet(&alias).unwrap();
            controller
                .cast_vote_private(
                    &private_voter,
                    &vote_plan_def,
                    &proposal.external_id(),
                    favorable,
                    &mut ledger,
                    &mut rng,
                )
                .unwrap();
            private_voter.confirm_transaction();
            total_votes_per_proposal[i] += voting_power;
        }
    }

    // Proceed to tally
    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    // Get encrypted tally
    let mut alice = controller.wallet(ALICE).unwrap();

    let encrypted_tally = EncryptedVoteTally::new(vote_plan.to_id());
    let fragment = controller
        .fragment_factory()
        .vote_encrypted_tally(&alice, encrypted_tally);

    let parameters = ledger.parameters.clone();
    let date = ledger.date();

    // benchmark the creation of encrypted tally
    c.bench_function("vote_encrypted_tally", |b| {
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
            let vote_plan: VotePlan = vote_plan.clone().into();
            c_vote_plan.id == vote_plan.to_id()
        })
        .unwrap();
    c.bench_function("tally_decrypt_share", |b| {
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
        let table = chain_vote::TallyOptimizationTable::generate_with_balance(total_votes, 1);
        vote_plan_status
            .proposals
            .iter()
            .enumerate()
            .map(|(i, proposal)| {
                let tally_state = proposal
                    .tally
                    .clone()
                    .unwrap()
                    .private_encrypted()
                    .unwrap()
                    .0
                    .state();
                chain_vote::tally(
                    total_votes_per_proposal[i],
                    &tally_state,
                    &decrypt_shares[i],
                    &table,
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    };

    c.bench_function("decrypt_private_tally", |b| b.iter(decrypt_tally));

    let shares = decrypt_tally()
        .into_iter()
        .zip(decrypt_shares.into_iter())
        .map(|(tally, decrypt_shares)| DecryptedPrivateTallyProposal {
            decrypt_shares: decrypt_shares.into_boxed_slice(),
            tally_result: tally.votes.into_boxed_slice(),
        })
        .collect();

    let decrypted_tally =
        VoteTally::new_private(vote_plan.to_id(), DecryptedPrivateTally::new(shares));
    let fragment = controller
        .fragment_factory()
        .vote_tally(&alice, decrypted_tally);

    c.bench_function("vote_tally", |b| {
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
    voters_count: usize,
    voting_power_per_voter: u64,
    c: &mut Criterion,
) {
    let voting_power_distribution = rand::distributions::uniform::Uniform::from(
        voting_power_per_voter..voting_power_per_voter + 1,
    );
    tally_benchmark(1, voters_count, 1.0, voting_power_distribution, c);
}

fn tally_benchmark_128_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution(128, 1000, c);
}

fn tally_benchmark_200_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution(200, 1000, c);
}

fn tally_benchmark_200_voters_1_000_000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution(200, 1_000_000, c);
}

fn tally_benchmark_1000_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark_flat_distribution(1000, 1000, c);
}

criterion_group!(
    benches,
    tally_benchmark_128_voters_1000_ada,
    tally_benchmark_200_voters_1000_ada,
    tally_benchmark_200_voters_1_000_000_ada,
    tally_benchmark_1000_voters_1000_ada,
);
criterion_main!(benches);
