#![cfg(test)]

use crate::{
    certificate::{PoolRegistration, PoolUpdate},
    date::BlockDate,
    fee::LinearFee,
    ledger::ledger::{Block0Error, Error},
    rewards::TaxType,
    stake::PoolError::NotFound,
    testing::{
        builders::{
            build_stake_pool_update_cert, create_initial_stake_pool_registration,
            create_initial_stake_pool_update, StakePoolBuilder, TestTxCertBuilder,
        },
        data::Wallet,
        ConfigBuilder, LedgerBuilder,
    },
    transaction::BalanceError::NotBalanced,
    value::*,
};

#[test]
pub fn pool_update_is_not_allowed_in_block0() {
    let alice = Wallet::from_value(Value(100));

    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    let registration_certificate =
        create_initial_stake_pool_registration(&stake_pool, &[alice.clone()]);

    let pool_update = make_minor_pool_update(&stake_pool.info());
    let update_certificate = create_initial_stake_pool_update(&pool_update, &[alice.clone()]);

    let ledger_builder_result = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[registration_certificate, update_certificate])
        .build();

    assert_eq!(
        ledger_builder_result.err().unwrap(),
        Error::Block0(Block0Error::HasPoolManagement)
    );
}

fn make_minor_pool_update(pool: &PoolRegistration) -> PoolUpdate {
    let mut new_pool_registration = pool.clone();
    new_pool_registration.serial = 123u128;

    PoolUpdate {
        pool_id: new_pool_registration.to_id(),
        last_pool_reg_hash: pool.to_id(),
        new_pool_reg: new_pool_registration,
    }
}

#[test]
pub fn pool_update_wrong_last_hash() {
    let alice = Wallet::from_value(Value(100));

    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    let registration_certificate =
        create_initial_stake_pool_registration(&stake_pool, &[alice.clone()]);
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[registration_certificate])
        .build()
        .unwrap();

    let mut new_pool_registration = stake_pool.clone();
    new_pool_registration.info_mut().serial = 1231u128;

    let pool_update = PoolUpdate {
        pool_id: stake_pool.id(),
        last_pool_reg_hash: new_pool_registration.info().to_id(),
        new_pool_reg: new_pool_registration.info(),
    };
    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, test_ledger.fee())
        .make_transaction(&[&alice], &certificate);

    assert_eq!(
        test_ledger
            .apply_fragment(&fragment, BlockDate::first())
            .err()
            .unwrap(),
        Error::PoolUpdateLastHashDoesntMatch
    );
}

#[test]
pub fn pool_update_not_enough_fee() {
    let alice = Wallet::from_value(Value(100));

    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    let registration_certificate =
        create_initial_stake_pool_registration(&stake_pool, &[alice.clone()]);
    let mut test_ledger =
        LedgerBuilder::from_config(ConfigBuilder::new(0).with_fee(LinearFee::new(1, 2, 3)))
            .faucets_wallets(vec![&alice])
            .certs(&[registration_certificate])
            .build()
            .unwrap();

    let mut new_pool_registration = stake_pool.clone();
    new_pool_registration.info_mut().serial = 1231u128;

    let pool_update = PoolUpdate {
        pool_id: stake_pool.id(),
        last_pool_reg_hash: stake_pool.info().to_id(),
        new_pool_reg: new_pool_registration.info(),
    };
    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, LinearFee::new(0, 0, 0))
        .make_transaction(&[&alice], &certificate);

    assert_eq!(
        test_ledger
            .apply_fragment(&fragment, BlockDate::first())
            .err()
            .unwrap(),
        Error::TransactionBalanceInvalid(NotBalanced {
            inputs: Value(0),
            outputs: Value(6)
        })
    );
}

#[test]
pub fn pool_update_wrong_pool_id() {
    let alice = Wallet::from_value(Value(100));

    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    let registration_certificate =
        create_initial_stake_pool_registration(&stake_pool, &[alice.clone()]);
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[registration_certificate])
        .build()
        .unwrap();

    let mut new_pool_registration = stake_pool.clone();
    new_pool_registration.info_mut().serial = 1231u128;

    let wrong_pool_id = new_pool_registration.info().to_id();

    let pool_update = PoolUpdate {
        pool_id: wrong_pool_id.clone(),
        last_pool_reg_hash: stake_pool.info().to_id(),
        new_pool_reg: new_pool_registration.info(),
    };
    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, test_ledger.fee())
        .make_transaction(&[&alice], &certificate);

    assert_eq!(
        test_ledger
            .apply_fragment(&fragment, BlockDate::first())
            .err()
            .unwrap(),
        Error::Delegation(NotFound(wrong_pool_id))
    );
}

#[test]
pub fn pool_update_use_old_hash() {
    let mut alice = Wallet::from_value(Value(100));

    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    let registration_certificate =
        create_initial_stake_pool_registration(&stake_pool, &[alice.clone()]);
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[registration_certificate])
        .build()
        .unwrap();

    let orginal_serial = stake_pool.info().serial;

    // first update

    let mut first_stake_pool_state = stake_pool.clone();
    first_stake_pool_state.info_mut().serial = 111u128;

    let mut pool_update = PoolUpdate {
        pool_id: stake_pool.id(),
        last_pool_reg_hash: stake_pool.info().to_id(),
        new_pool_reg: first_stake_pool_state.info(),
    };

    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, test_ledger.fee())
        .make_transaction(&[&alice], &certificate);

    assert!(test_ledger
        .apply_fragment(&fragment, BlockDate::first())
        .is_ok());
    alice.confirm_transaction();
    // revert

    let mut reverted_stake_pool_state = first_stake_pool_state.clone();
    reverted_stake_pool_state.info_mut().serial = orginal_serial;

    pool_update = PoolUpdate {
        pool_id: stake_pool.id(),
        last_pool_reg_hash: first_stake_pool_state.info().to_id(),
        new_pool_reg: reverted_stake_pool_state.info(),
    };

    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, test_ledger.fee())
        .make_transaction(&[&alice], &certificate);

    assert!(test_ledger
        .apply_fragment(&fragment, BlockDate::first())
        .is_ok());
    alice.confirm_transaction();

    // try to set serial again using orginal info hash

    let mut last_pool_state = stake_pool.clone();
    last_pool_state.info_mut().serial = 222u128;

    pool_update = PoolUpdate {
        pool_id: stake_pool.id(),
        last_pool_reg_hash: stake_pool.info().to_id(),
        new_pool_reg: last_pool_state.info(),
    };

    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, test_ledger.fee())
        .make_transaction(&[&alice], &certificate);

    assert!(test_ledger
        .apply_fragment(&fragment, BlockDate::first())
        .is_ok());
}

#[test]
pub fn pool_update_update_fee_is_not_allowed() {
    let alice = Wallet::from_value(Value(100));

    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    let registration_certificate =
        create_initial_stake_pool_registration(&stake_pool, &[alice.clone()]);
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[registration_certificate])
        .build()
        .unwrap();

    let mut new_pool_registration = stake_pool.clone();
    new_pool_registration.info_mut().rewards = TaxType::zero();

    let pool_update = PoolUpdate {
        last_pool_reg_hash: stake_pool.id(),
        pool_id: stake_pool.id(),
        new_pool_reg: new_pool_registration.info(),
    };

    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, test_ledger.fee())
        .make_transaction(&[&alice], &certificate);

    assert_eq!(
        test_ledger
            .apply_fragment(&fragment, BlockDate::first())
            .err()
            .unwrap(),
        Error::PoolUpdateFeesNotAllowedYet
    );
}

#[test]
pub fn pool_update_without_any_change() {
    let alice = Wallet::from_value(Value(100));

    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    let registration_certificate =
        create_initial_stake_pool_registration(&stake_pool, &[alice.clone()]);
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .faucets_wallets(vec![&alice])
        .certs(&[registration_certificate])
        .build()
        .unwrap();

    let pool_update = PoolUpdate {
        pool_id: stake_pool.info().to_id(),
        last_pool_reg_hash: stake_pool.info().to_id(),
        new_pool_reg: stake_pool.info(),
    };
    let certificate = build_stake_pool_update_cert(&pool_update);
    let fragment = TestTxCertBuilder::new(test_ledger.block0_hash, test_ledger.fee())
        .make_transaction(&[&alice], &certificate);

    assert!(test_ledger
        .apply_fragment(&fragment, BlockDate::first())
        .is_ok());
}
