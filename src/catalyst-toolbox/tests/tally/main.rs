mod blockchain;

mod generator;

use chain_addr::Discrimination;
use chain_impl_mockchain::accounting::account::SpendingCounter;
pub use chain_impl_mockchain::chaintypes::ConsensusVersion;
use chain_impl_mockchain::{
    certificate::VoteTallyPayload,
    fragment::Fragment,
    vote::{Choice, PayloadType, VotePlanStatus},
};
use generator::{TestStrategy, VoteRoundGenerator};
use jormungandr_lib::{
    interfaces::{
        Block0Configuration, FragmentLogDeserializeError, Initial, PersistentFragmentLog,
    },
    time::SecondsSinceUnixEpoch,
};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use std::collections::HashMap;
use thor::{FragmentBuilder, Wallet};

fn jump_to_epoch(epoch: u64, block0_config: &Block0Configuration) -> SecondsSinceUnixEpoch {
    let slots_per_epoch: u32 = block0_config
        .blockchain_configuration
        .slots_per_epoch
        .into();
    let slot_duration: u8 = block0_config.blockchain_configuration.slot_duration.into();
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
        in_order = $in_order:expr,
        payload = $payload:expr
    ) => {
        {
            let seed = $seed;
            let mut rng = ChaChaRng::from_seed(seed);
            let blockchain = blockchain::TestBlockchainBuilder::new()
                $(
                    .with_voteplan($vote_start, $vote_end, $committe_eend, $proposals, $payload)
                )*
                $(
                    .with_n_wallets($wallets)
                )?
                .build(&mut rng);
            let mut generator = VoteRoundGenerator::new(blockchain);
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
            let tally_fragments = generator.tally_transactions(&mut rng).into_iter().map(|fragment| {
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
    let (generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 255 proposals
            ]
        ],
        votes = 1,
        in_order = true,
        payload = PayloadType::Public
    };

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.statuses());
    assert!(failed_fragments.is_empty());
}

//TV 002
#[test]
fn shuffle_tally_ok() {
    let (generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 255 proposals
            ]
        ],
        votes = 1000,
        in_order = false,
        payload = PayloadType::Public
    };

    let (ledger, _) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.statuses());
}

#[test]
fn shuffle_tally_ok_private() {
    let (generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 255 proposals
            ]
        ],
        votes = 1000,
        in_order = false,
        payload = PayloadType::Private
    };

    let (ledger, _) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.statuses());
}

//TV 003
#[test]
fn wallet_not_in_block0() {
    let (generator, vote_fragments, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 1 proposals
            ]
        ],
        votes = 0,
        in_order = false,
        payload = PayloadType::Public
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

    assert_tally_eq(ledger.active_vote_plans(), generator.statuses());
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
        in_order = false,
        payload = PayloadType::Public
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

    let tally = ledger.active_vote_plans()[0].proposals[0].tally.clone();
    dbg!(&tally);
    assert_eq!(tally.result().unwrap().results()[0], 0.into());
    assert_eq!(tally.result().unwrap().results()[2], 0.into());
    assert!(tally.result().unwrap().results()[1] > 0.into());
    assert_eq!(failed_fragments.len(), 2);
}

//TV 005
#[test]
fn replay_not_counted() {
    let (mut generator, _, tally_fragments) = setup_run! {
        seed = [0; 32],
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 2 proposals
            ]
        ],
        votes = 0,
        in_order = false,
        payload = PayloadType::Public
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

    let tally = ledger.active_vote_plans()[0].proposals[0].tally.clone();
    dbg!(&tally);
    assert_eq!(tally.result().unwrap().results()[0], 0.into());
    assert!(tally.result().unwrap().results()[1] > 0.into());
    assert_eq!(tally.result().unwrap().results()[2], 0.into());
    assert_eq!(failed_fragments.len(), 2);
}

//TV 006
#[test]
fn multi_voteplan_ok() {
    let (generator, vote_fragments, tally_fragments) = setup_run! {
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
        in_order = false,
        payload = PayloadType::Public
    };

    let (ledger, _) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.statuses());
}

#[test]
fn multi_voteplan_ok_private() {
    let (generator, vote_fragments, tally_fragments) = setup_run! {
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
        in_order = false,
        payload = PayloadType::Private
    };

    let (ledger, _) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vote_fragments
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.statuses());
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
        in_order = false,
        payload = PayloadType::Public
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

    let mut committee_wallet = generator
        .committee_wallets()
        .values()
        .next()
        .unwrap()
        .clone();
    committee_wallet.update_counter(SpendingCounter::zero());

    let early_tally_fragment = FragmentBuilder::new(
        &generator.block0().header().id().into(),
        &generator
            .block0_config()
            .blockchain_configuration
            .linear_fees,
        generator.voteplans()[0].vote_end(),
    )
    .vote_tally(
        &committee_wallet,
        generator.voteplans()[0],
        VoteTallyPayload::Public,
    );

    let early_tally = Ok(PersistentFragmentLog {
        fragment: early_tally_fragment,
        time: jump_to_epoch(0, generator.block0_config()),
    });

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &block0,
        vec![fragment_yes, early_tally, fragment_no]
            .into_iter()
            .chain(tally_fragments),
    )
    .unwrap();

    let tally = ledger.active_vote_plans()[0].proposals[0].tally.clone();
    dbg!(&tally);
    assert_eq!(tally.result().unwrap().results()[0], 0.into());
    assert_eq!(tally.result().unwrap().results()[1], 0.into());
    assert_eq!(tally.result().unwrap().results()[2], 0.into());
    assert_eq!(failed_fragments.len(), 3);
}

#[test]
fn transaction_transfer_does_not_decrease_voting_power() {
    let (mut generator, _, tally_fragments) = setup_run! {
        seed = [0; 32],
        wallets = 4,
        voteplans = [
            dates 0 => 1 => 2,
            plans = [
                one with 1 proposals
            ]
        ],
        votes = 0,
        in_order = false,
        payload = PayloadType::Public
    };
    let mut wallets = generator
        .wallets()
        .values()
        .take(2)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(wallets.len(), 2);
    let wallet1_address = wallets[1].address();
    let wallet0_address = wallets[0].address();

    let config = generator.block0_config();
    let wallets_stake = config
        .initial
        .iter()
        .filter_map(|initial| {
            if let Initial::Fund(utxos) = initial {
                Some(utxos.iter())
            } else {
                None
            }
        })
        .flatten()
        .map(|utxo| (utxo.address.clone(), utxo.value))
        .filter(|(addr, _)| [&wallet0_address, &wallet1_address].contains(&addr))
        .collect::<HashMap<_, _>>();

    // transfer all funds from the first wallet to the second
    let transaction = FragmentBuilder::new(
        &generator.block0().header().id().into(),
        &generator
            .block0_config()
            .blockchain_configuration
            .linear_fees,
        generator.voteplans()[0].vote_start(),
    )
    .transaction(
        &wallets[0],
        wallet1_address.clone(),
        *wallets_stake.get(&wallet0_address).unwrap(),
    )
    .unwrap();

    let transaction = Ok(PersistentFragmentLog {
        fragment: transaction,
        time: SecondsSinceUnixEpoch::now(),
    });

    let fragment_yes = Ok(PersistentFragmentLog {
        fragment: cast_vote(&mut wallets[0], &generator, 0, 1),
        time: SecondsSinceUnixEpoch::now(),
    });
    let fragment_no = Ok(PersistentFragmentLog {
        fragment: cast_vote(&mut wallets[1], &generator, 0, 2),
        time: SecondsSinceUnixEpoch::now(),
    });

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vec![transaction, fragment_yes, fragment_no]
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    let tally = ledger.active_vote_plans()[0].proposals[0].tally.clone();

    let wallet0_weight: u64 = wallets_stake[&wallet0_address].into();
    let wallet1_weight: u64 = wallets_stake[&wallet1_address].into();
    assert_eq!(tally.result().unwrap().results()[0], 0.into());
    // the first wallet has the same voting power as before
    assert_eq!(tally.result().unwrap().results()[1], wallet0_weight.into());
    // the second wallet should have also the same voting power
    assert_eq!(tally.result().unwrap().results()[2], wallet1_weight.into());

    // No fragments are rejected because there are no fees in the current configuration, otherwise the first wallet
    // would not have enough funds for the vote cast transaction
    assert!(failed_fragments.is_empty());
}

#[test]
fn expired_transaction() {
    let (mut generator, _, tally_fragments) = setup_run! {
        seed = [0; 32],
        wallets = 1000,
        voteplans = [
            dates 0 => 2 => 3,
            plans = [
                one with 255 proposals
            ]
        ],
        votes = 0,
        in_order = false,
        payload = PayloadType::Private
    };
    let mut wallet = generator.wallets().values().next().unwrap().clone();

    let fragment_yes = Ok(PersistentFragmentLog {
        fragment: cast_vote(&mut wallet, &generator, 0, 1),
        time: jump_to_epoch(2, generator.block0_config()),
    });

    let (ledger, failed_fragments) = catalyst_toolbox::recovery::tally::recover_ledger_from_logs(
        &generator.block0(),
        vec![fragment_yes]
            .into_iter()
            .chain(tally_fragments.into_iter()),
    )
    .unwrap();

    assert_tally_eq(ledger.active_vote_plans(), generator.statuses());
    assert_eq!(failed_fragments.len(), 1);
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
    FragmentBuilder::new(
        &generator.block0().header().id().into(),
        &generator
            .block0_config()
            .blockchain_configuration
            .linear_fees,
        generator.voteplans()[0].vote_start().next_epoch(),
    )
    .vote_cast(
        wallet,
        generator.voteplans()[0],
        proposals_idx,
        &Choice::new(choice),
    )
}
