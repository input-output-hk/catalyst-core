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

use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

const ALICE: &str = "Alice";
const STAKE_POOL: &str = "stake_pool";
const VOTE_PLAN: &str = "fund1";

fn voters_aliases(count: usize) -> Vec<String> {
    let mut counter = 0;
    std::iter::from_fn(|| {
        counter += 1;
        Some(format!("voter_{}", counter))
    })
    .take(count)
    .collect()
}

fn tally_benchmark(voters_count: usize, voting_power_per_voter: u64, c: &mut Criterion) {
    const MEMBERS_NO: usize = 3;
    const THRESHOLD: usize = 2;
    let favorable = Choice::new(1);

    let mut wallets: Vec<&mut WalletTemplateBuilder> = Vec::new();

    let mut alice_wallet_builder = wallet(ALICE);
    alice_wallet_builder
        .with(1_000)
        .owns(STAKE_POOL)
        .committee_member();
    wallets.push(&mut alice_wallet_builder);

    let voters_aliases = voters_aliases(voters_count);
    let mut wallet_builders: Vec<WalletTemplateBuilder> =
        voters_aliases.iter().map(|alias| wallet(alias)).collect();

    for wallet_builder in wallet_builders.iter_mut() {
        wallet_builder.with(voting_power_per_voter);
        wallets.push(wallet_builder);
    }

    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let members = CommitteeMembersManager::new(&mut rng, THRESHOLD, MEMBERS_NO);

    let committee_keys = members
        .members()
        .iter()
        .map(|committee_member| committee_member.public_key())
        .collect::<Vec<_>>();

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_fee(LinearFee::new(0, 0, 0))
                .with_rewards(Value(1000)),
        )
        .with_initials(wallets)
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .payload_type(PayloadType::Private)
            .committee_keys(committee_keys)
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_transfer_to_rewards(100),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();

    let vote_plan_def = controller.vote_plan(VOTE_PLAN).unwrap();
    let vote_plan: VotePlan = vote_plan_def.clone().into();
    let proposal = vote_plan_def.proposal(0);

    for alias in voters_aliases {
        let mut private_voter = controller.wallet(&alias).unwrap();

        controller
            .cast_vote_private(
                &private_voter,
                &vote_plan_def,
                &proposal.id(),
                favorable,
                &mut ledger,
                &mut rng,
            )
            .unwrap();
        private_voter.confirm_transaction();
    }

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    let encrypted_tally = EncryptedVoteTally::new(vote_plan.to_id());
    let fragment = controller
        .fragment_factory()
        .vote_encrypted_tally(&alice, encrypted_tally);

    let parameters = ledger.parameters.clone();
    let date = ledger.date();

    let bench_name_suffix = format!("_{}_voters_{}_ada", voters_count, voting_power_per_voter);

    c.bench_function(&format!("vote_encrypted_tally{}", bench_name_suffix), |b| {
        b.iter(|| {
            ledger
                .ledger
                .apply_fragment(&parameters, &fragment, date)
                .unwrap();
        })
    });

    ledger.apply_fragment(&fragment, ledger.date()).unwrap();
    alice.confirm_transaction();

    let vote_plans = ledger.ledger.active_vote_plans();
    let vote_plan_status = vote_plans
        .iter()
        .find(|c_vote_plan| {
            let vote_plan: VotePlan = vote_plan.clone().into();
            c_vote_plan.id == vote_plan.to_id()
        })
        .unwrap();

    c.bench_function(&format!("tally_decrypt_share{}", bench_name_suffix), |b| {
        b.iter(|| {
            members.members()[0].produce_decrypt_shares(&vote_plan_status);
        })
    });

    let decrypt_shares: Vec<_> = members
        .members()
        .iter()
        // We use only one proposal in this benchmark so here's a bit of a dirty hack.
        .map(|member| member.produce_decrypt_shares(&vote_plan_status).into_iter())
        .flatten()
        .collect();

    let decrypt_tally = || {
        let total_votes = voters_count as u64 * voting_power_per_voter;
        let tally_state = vote_plan_status.proposals[0]
            .tally
            .clone()
            .unwrap()
            .private_encrypted()
            .unwrap()
            .0
            .state();
        let table = chain_vote::TallyOptimizationTable::generate_with_balance(total_votes, 1);
        chain_vote::tally(total_votes, &tally_state, &decrypt_shares, &table).unwrap()
    };

    c.bench_function(
        &format!("decrypt_private_tally{}", bench_name_suffix),
        |b| b.iter(decrypt_tally),
    );

    let tally = decrypt_tally();
    let shares = DecryptedPrivateTallyProposal {
        decrypt_shares: decrypt_shares.into_boxed_slice(),
        tally_result: tally.votes.into_boxed_slice(),
    };

    let decrypted_tally =
        VoteTally::new_private(vote_plan.to_id(), DecryptedPrivateTally::new(vec![shares]));
    let fragment = controller
        .fragment_factory()
        .vote_tally(&alice, decrypted_tally);

    c.bench_function(&format!("vote_tally{}", bench_name_suffix), |b| {
        b.iter(|| {
            ledger
                .ledger
                .apply_fragment(&parameters, &fragment, ledger.date())
                .unwrap();
        })
    });

    ledger.apply_fragment(&fragment, ledger.date()).unwrap();
}

fn tally_benchmark_128_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark(128, 1000, c);
}

fn tally_benchmark_200_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark(200, 1000, c);
}

fn tally_benchmark_200_voters_1_000_000_ada(c: &mut Criterion) {
    tally_benchmark(200, 1_000_000, c);
}

fn tally_benchmark_1000_voters_1000_ada(c: &mut Criterion) {
    tally_benchmark(1000, 1000, c);
}

criterion_group!(
    benches,
    tally_benchmark_128_voters_1000_ada,
    tally_benchmark_200_voters_1000_ada,
    tally_benchmark_200_voters_1_000_000_ada,
    tally_benchmark_1000_voters_1000_ada,
);
criterion_main!(benches);
