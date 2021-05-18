mod blockchain;

mod generator;

use catalyst_toolbox;
use chain_addr::Discrimination;
pub use chain_impl_mockchain::chaintypes::ConsensusVersion;
use chain_impl_mockchain::{
    fragment::Fragment,
    vote::{Choice, VotePlanStatus},
};
use generator::{TestStrategy, VoteRoundGenerator};
use jormungandr_lib::{
    interfaces::{Block0Configuration, FragmentLogDeserializeError, PersistentFragmentLog},
    time::SecondsSinceUnixEpoch,
};
use jormungandr_testing_utils::wallet::Wallet;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

fn jump_to_epoch(epoch: u64, block0_config: &Block0Configuration) -> SecondsSinceUnixEpoch {
    let slots_per_epoch: u32 = block0_config
        .blockchain_configuration
        .slots_per_epoch
        .into();
    let slot_duration: u8 = block0_config
        .blockchain_configuration
        .slot_duration
        .clone()
        .into();
    SecondsSinceUnixEpoch::from_secs(
        SecondsSinceUnixEpoch::now().to_secs()
            + slot_duration as u64 * slots_per_epoch as u64 * epoch,
    )
}

macro_rules! setup_run {
    (
        seed = $seed:expr,
        $(wallets = $wallets:tt)? $(,)?
        voteplans = [
            dates $vote_start:expr => $vote_end:expr => $committe_eend:tt,
            plans = [
                $(one with $proposals:tt proposals),+
            ]
        ],
        votes = $votes:expr,
        in_order = $in_order:expr
    ) => {
        {
            let seed = $seed;
            let mut rng = ChaChaRng::from_seed(seed);
            let blockchain = blockchain::BlockchainBuilder::new()
                $(
                    .with_public_voteplan($vote_start, $vote_end, $committe_eend, $proposals)
                )*
                $(
                    .with_n_wallets($wallets)
                )?
                .build(&mut rng);
            let mut generator = VoteRoundGenerator::new(blockchain, &mut rng);
            let vote_end = jump_to_epoch($vote_end, generator.block0_config());
            let vote_fragments = generator
                .generate_vote_fragments(TestStrategy::Random(seed), $votes, !$in_order, &mut rng)
                .into_iter()
                .map(|fragment| {
                    Ok::<PersistentFragmentLog, FragmentLogDeserializeError>(PersistentFragmentLog {
                        time: jump_to_epoch($vote_start, generator.block0_config()),
                        fragment,
                    })
                }).collect::<Vec<_>>();
            let tally_fragments = generator.tally_transactions().into_iter().map(|fragment| {
                Ok(PersistentFragmentLog {
                    time: vote_end,
                    fragment,
                })
            }).collect::<Vec<_>>();
            (generator, vote_fragments, tally_fragments)
        }
    };
}

//TV 001
#[test]
fn tally_ok() {
    let (mut generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 255 proposals
            ]
        ],
        votes = 1000,
        in_order = true
    };

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.tally());
    assert!(failed_fragments.is_empty());
}

//TV 002
#[test]
fn shuffle_tally_ok() {
    let (mut generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 255 proposals
            ]
        ],
        votes = 1000,
        in_order = false
    };

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.tally());
    assert!(failed_fragments.is_empty());
}

//TV 003
#[test]
fn wallet_not_in_block0() {
    let (mut generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 1 proposals
            ]
        ],
        votes = 0,
        in_order = false
    };

    let block0 = generator.block0();
    let mut rng = ChaChaRng::from_seed([0; 32]);

    let fragment = cast_vote(
        &mut Wallet::new_account_with_discrimination(&mut rng, Discrimination::Production),
        &generator,
        0,
        1,
    );

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &block0,
        vote_fragments
            .into_iter()
            .chain(std::iter::once(Ok(PersistentFragmentLog {
                time: jump_to_epoch(0, generator.block0_config()),
                fragment,
            })))
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.tally());
    assert_eq!(failed_fragments.len(), 1);
}

//TV 004
#[test]
fn only_last_vote_is_counted() {
    let (mut generator, _, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 1 proposals
            ]
        ],
        votes = 0,
        in_order = false
    };

    let block0 = generator.block0();

    let mut wallet = generator.wallets().values().next().unwrap().clone();
    let fragment_yes_1 = cast_vote(&mut wallet, &generator, 0, 1);
    wallet.confirm_transaction();

    let fragment_no = cast_vote(&mut wallet, &generator, 0, 2);
    wallet.confirm_transaction();

    let fragment_yes_2 = cast_vote(&mut wallet, &generator, 0, 1);
    wallet.confirm_transaction();

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &block0,
        vec![fragment_yes_1, fragment_no, fragment_yes_2]
            .into_iter()
            .map(|fragment| {
                Ok(PersistentFragmentLog {
                    time: jump_to_epoch(0, generator.block0_config()),
                    fragment,
                })
            })
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    let tally = ledger.active_vote_plans()[0].proposals[0]
        .tally
        .clone()
        .unwrap();
    dbg!(&tally);
    assert_eq!(tally.result().unwrap().results()[0], 0.into());
    assert_eq!(tally.result().unwrap().results()[2], 0.into());
    assert!(tally.result().unwrap().results()[1] > 0.into());
    assert!(failed_fragments.is_empty());
}

//TV 005
#[test]
fn replay_not_counted() {
    let (mut generator, _, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 1 proposals
            ]
        ],
        votes = 0,
        in_order = false
    };

    let block0 = generator.block0();

    let mut wallet = generator.wallets().values().next().unwrap().clone();
    let fragment_yes = cast_vote(&mut wallet, &generator, 0, 1);
    wallet.confirm_transaction();

    let fragment_no = cast_vote(&mut wallet, &generator, 0, 2);
    wallet.confirm_transaction();

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &block0,
        vec![fragment_yes.clone(), fragment_no, fragment_yes]
            .into_iter()
            .map(|fragment| {
                Ok(PersistentFragmentLog {
                    time: jump_to_epoch(0, generator.block0_config()),
                    fragment,
                })
            })
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    let tally = ledger.active_vote_plans()[0].proposals[0]
        .tally
        .clone()
        .unwrap();
    dbg!(&tally);
    assert_eq!(tally.result().unwrap().results()[0], 0.into());
    assert_eq!(tally.result().unwrap().results()[1], 0.into());
    assert!(tally.result().unwrap().results()[2] > 0.into());
    assert_eq!(failed_fragments.len(), 1);
}

//TV 006
#[test]
fn multi_voteplan_ok() {
    let (mut generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        wallets = 1000,
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 255 proposals,
                one with 255 proposals,
                one with 255 proposals,
                one with 255 proposals
            ]
        ],
        votes = 10000,
        in_order = false
    };

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.tally());
    assert!(failed_fragments.is_empty());
}

#[test]
fn votes_outside_voting_phase() {
    let (mut generator, _, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 1 proposals
            ]
        ],
        votes = 0,
        in_order = false
    };

    let block0 = generator.block0();

    let mut wallet = generator.wallets().values().next().unwrap().clone();
    let fragment_yes = Ok(PersistentFragmentLog {
        fragment: cast_vote(&mut wallet, &generator, 0, 1),
        time: SecondsSinceUnixEpoch::from_secs(0),
    });
    wallet.confirm_transaction();
    let fragment_no = Ok(PersistentFragmentLog {
        fragment: cast_vote(&mut wallet, &generator, 0, 2),
        time: jump_to_epoch(1, generator.block0_config()),
    });

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &block0,
        vec![fragment_yes, fragment_no]
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    let tally = ledger.active_vote_plans()[0].proposals[0]
        .tally
        .clone()
        .unwrap();
    dbg!(&tally);
    assert_eq!(tally.result().unwrap().results()[0], 0.into());
    assert_eq!(tally.result().unwrap().results()[1], 0.into());
    assert_eq!(tally.result().unwrap().results()[2], 0.into());
    assert_eq!(failed_fragments.len(), 2);
}

fn assert_tally_eq(mut r1: Vec<VotePlanStatus>, mut r2: Vec<VotePlanStatus>) {
    r1.sort_by_key(|plan| plan.id.clone());
    r2.sort_by_key(|plan| plan.id.clone());

    for (plan1, plan2) in r1.into_iter().zip(r2.into_iter()) {
        assert_eq!(plan1.proposals.len(), plan2.proposals.len());
        for (p1, p2) in plan1.proposals.into_iter().zip(plan2.proposals.into_iter()) {
            assert_eq!(p1.proposal_id, p2.proposal_id);
            assert_eq!(p2.tally, p1.tally);
        }
    }
}

fn cast_vote(
    wallet: &mut Wallet,
    generator: &VoteRoundGenerator,
    proposals_idx: u8,
    choice: u8,
) -> Fragment {
    wallet
        .issue_vote_cast_cert(
            &generator.block0().header.id().clone().into(),
            &generator
                .block0_config()
                .blockchain_configuration
                .linear_fees,
            &generator.voteplans()[0],
            proposals_idx,
            &Choice::new(choice),
        )
        .unwrap()
}
