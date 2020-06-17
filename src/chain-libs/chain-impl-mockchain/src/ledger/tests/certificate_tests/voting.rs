#![cfg(test)]

use crate::{
    testing::{
        builders::{
            create_initial_vote_cast, create_initial_vote_plan, InitialFaultTolerantTxCertBuilder,
        },
        data::Wallet,
        ConfigBuilder, LedgerBuilder, VoteTestGen,
    },
    key::BftLeaderId,
    value::*,
};

#[test]
pub fn vote_plan_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let vote_plan = VoteTestGen::vote_plan();
    let vote_plan_certificate = create_initial_vote_plan(&vote_plan, &[alice.clone()]);

    let leader = BftLeaderId::from(alice.public_key());
    let config_builder = ConfigBuilder::new(0).with_leaders(&[leader]);

    LedgerBuilder::from_config(config_builder)
        .faucets_wallets(vec![&alice])
        .certs(&[vote_plan_certificate])
        .build()
        .expect("ledger should be built with vote plan certificate");
}

#[ignore]
#[test]
pub fn vote_plan_in_block0_with_input() {
    let alice = Wallet::from_value(Value(100));
    let vote_plan = VoteTestGen::vote_plan();
    let vote_plan_certificate =
        InitialFaultTolerantTxCertBuilder::new(vote_plan.into(), alice.clone())
            .transaction_with_input_only();

    let ledger_build_result = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[vote_plan_certificate])
        .build();

    assert!(
        ledger_build_result.is_err(),
        "ledger should not be built with faulty vote plan certificate"
    );
}

#[test]
pub fn vote_plan_in_block0_with_output() {
    let alice = Wallet::from_value(Value(100));
    let vote_plan = VoteTestGen::vote_plan();
    let vote_plan_certificate =
        InitialFaultTolerantTxCertBuilder::new(vote_plan.into(), alice.clone())
            .transaction_with_output_only();

    let ledger_build_result = LedgerBuilder::from_config(ConfigBuilder::new(0))
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

    let ledger_build_result = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[vote_plan_certificate, vote_cast_certificate])
        .build();

    assert!(
        ledger_build_result.is_err(),
        "ledger should not be built with faulty vote plan certificate"
    );
}
