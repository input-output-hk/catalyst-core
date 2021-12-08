#![cfg(test)]

use crate::{
    key::BftLeaderId,
    testing::{builders::*, data::Wallet, ConfigBuilder, LedgerBuilder, TestGen},
    value::*,
};

#[test]
pub fn mint_token_in_block0() {
    let alice = Wallet::from_value(Value(100));
    let token_certificate =
        create_initial_mint_token(TestGen::mint_token_for_wallet(alice.public_key().into()));

    let leader = BftLeaderId::from(alice.public_key());
    let config_builder = ConfigBuilder::new().with_leaders(&[leader]);

    LedgerBuilder::from_config(config_builder)
        .faucets_wallets(vec![&alice])
        .certs(&[token_certificate])
        .build()
        .expect("ledger should be built with mint_token certificate");
}

#[test]
pub fn mint_token_in_block0_with_input() {
    let alice = Wallet::from_value(Value(100));
    let mint_token = TestGen::mint_token_for_wallet(alice.public_key().into());
    let mint_token_tx = InitialFaultTolerantTxCertBuilder::new(mint_token.into(), alice.clone())
        .transaction_with_input_only();

    let result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .certs(&[mint_token_tx])
        .build();

    assert!(result.is_err());
}

#[test]
pub fn mint_token_in_block0_with_output() {
    let alice = Wallet::from_value(Value(100));
    let mint_token = TestGen::mint_token_for_wallet(alice.public_key().into());
    let mint_token_tx = InitialFaultTolerantTxCertBuilder::new(mint_token.into(), alice.clone())
        .transaction_with_output_only();

    let ledger_build_result = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucets_wallets(vec![&alice])
        .certs(&[mint_token_tx])
        .build();

    assert!(
        ledger_build_result.is_err(),
        "ledger should not be built with faulty mint token certificate"
    );
}
