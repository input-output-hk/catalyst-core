#![cfg(test)]

use crate::{
    config::ConfigParam,
    key::BftLeaderId,
    ledger::ledger::{Block0Error, Error},
    testing::{
        builders::{
            create_initial_update_proposal, create_initial_update_vote, create_initial_vote_cast,
            create_initial_vote_plan, create_initial_vote_tally, InitialFaultTolerantTxCertBuilder,
        },
        data::Wallet,
        ConfigBuilder, LedgerBuilder, TestGen, VoteTestGen,
    },
    value::*,
};

#[test]
pub fn vote_plan_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let vote_plan = VoteTestGen::vote_plan();
    let vote_plan_certificate = create_initial_vote_plan(&vote_plan, &[alice.clone()]);

    let leader = BftLeaderId::from(alice.public_key());
    let config_builder = ConfigBuilder::new().with_leaders(&[leader]);

    LedgerBuilder::from_config(config_builder)
        .faucets_wallets(vec![&alice])
        .certs(&[vote_plan_certificate])
        .build()
        .expect("ledger should be built with vote plan certificate");
}

#[test]
pub fn vote_plan_in_block0_with_input() {
    let alice = Wallet::from_value(Value(100));
    let vote_plan = VoteTestGen::vote_plan();
    let vote_plan_certificate =
        InitialFaultTolerantTxCertBuilder::new(vote_plan.into(), alice.clone())
            .transaction_with_input_only();

    let result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .certs(&[vote_plan_certificate])
        .build();

    assert!(result.is_err());
}

#[test]
pub fn vote_plan_in_block0_with_output() {
    let alice = Wallet::from_value(Value(100));
    let vote_plan = VoteTestGen::vote_plan();
    let vote_plan_certificate =
        InitialFaultTolerantTxCertBuilder::new(vote_plan.into(), alice.clone())
            .transaction_with_output_only();

    let ledger_build_result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .certs(&[vote_plan_certificate])
        .build();

    assert!(
        ledger_build_result.is_err(),
        "ledger should not be built with faulty vote plan certificate"
    );
}

#[test]
pub fn vote_cast_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let vote_plan = VoteTestGen::vote_plan();
    let vote_cast = VoteTestGen::vote_cast_for(&vote_plan);
    let vote_plan_certificate = create_initial_vote_plan(&vote_plan, &[alice.clone()]);
    let vote_cast_certificate = create_initial_vote_cast(&vote_cast, &[alice.clone()]);

    let ledger_build_result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .certs(&[vote_plan_certificate, vote_cast_certificate])
        .build();

    assert!(
        ledger_build_result.is_err(),
        "ledger should not be built with faulty vote plan certificate"
    );
}

#[test]
pub fn vote_cast_is_not_allowed_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let vote_cast = VoteTestGen::vote_cast();
    let vote_cast_cert = create_initial_vote_cast(&vote_cast, &[alice.clone()]);

    let ledger_builder_result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .certs(&[vote_cast_cert])
        .build();

    assert_eq!(
        ledger_builder_result.err().unwrap(),
        Error::Block0(Block0Error::HasVoteCast)
    );
}

#[test]
pub fn vote_tally_is_not_allowed_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let vote_tally = VoteTestGen::vote_tally();

    let vote_tally_cert = create_initial_vote_tally(&vote_tally, &[alice.clone()]);

    let ledger_builder_result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .certs(&[vote_tally_cert])
        .build();

    assert_eq!(
        ledger_builder_result.err().unwrap(),
        Error::Block0(Block0Error::HasVoteTally)
    );
}

#[test]
pub fn update_vote_is_not_allowed_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let leader_pair = TestGen::leader_pair();

    let fragment = create_initial_update_vote(leader_pair, TestGen::hash());

    let ledger_builder_result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .fragment(fragment)
        .build();

    assert_eq!(
        ledger_builder_result.err().unwrap(),
        Error::Block0(Block0Error::HasUpdateVote)
    );
}

#[test]
pub fn update_proposal_is_not_allowed_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let leader_pair = TestGen::leader_pair();

    let fragment =
        create_initial_update_proposal(leader_pair, vec![ConfigParam::SlotDuration(11u8)]);

    let ledger_builder_result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .fragment(fragment)
        .build();

    assert_eq!(
        ledger_builder_result.err().unwrap(),
        Error::Block0(Block0Error::HasUpdateProposal)
    );
}
