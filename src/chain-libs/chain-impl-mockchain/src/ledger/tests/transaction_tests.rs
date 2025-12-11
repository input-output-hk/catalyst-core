#![cfg(test)]

use crate::{
    accounting::account::{LedgerError::NonExistent, SpendingCounter},
    ledger::{
        self,
        check::TxVerifyError,
        Error::{Account, TransactionMalformed},
    },
    testing::{
        data::{AddressData, AddressDataValue},
        ConfigBuilder, LedgerBuilder, TestTxBuilder,
    },
    transaction::*,
    value::*,
};
use chain_addr::Discrimination;

#[test]
pub fn transaction_fail_when_255_outputs() {
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucet_value(Value(1000))
        .build()
        .expect("cannot build test ledger");

    // random output repeated 255 times.
    let receiver = AddressData::utxo(Discrimination::Test);
    let output = Output {
        address: receiver.address,
        value: Value(1),
    };
    let outputs: Vec<_> = std::iter::repeat_n(output, 255).collect();

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_to_outputs_from_faucet(&mut test_ledger, &outputs)
        .get_fragment();

    assert_err!(
        TransactionMalformed(TxVerifyError::TooManyOutputs {
            expected: 254,
            actual: 255
        }),
        test_ledger.apply_transaction(fragment)
    );
}

#[test]
pub fn duplicated_account_transaction() {
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucet_value(Value(1000))
        .build()
        .expect("cannot build test ledger");

    let receiver = AddressData::utxo(Discrimination::Test);

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_from_faucet(&mut test_ledger, &receiver.address, Value(100))
        .get_fragment();
    let fragment2 = fragment.clone();
    let result = test_ledger.apply_transaction(fragment);

    match result {
        Err(err) => panic!("first transaction should be succesful but {}", err),
        Ok(_) => match test_ledger.apply_transaction(fragment2) {
            Err(ledger::Error::Account(crate::account::LedgerError::SpendingCounterError(
                crate::accounting::account::spending::Error::SpendingCredentialInvalid {
                    expected,
                    actual,
                },
            ))) => {
                assert_eq!(expected, SpendingCounter::zero().increment());
                assert_eq!(actual, SpendingCounter::zero());
            }
            _ => panic!("duplicated transaction should fail spending counter validation"),
        },
    }
}

#[test]
pub fn transaction_nonexisting_account_input() {
    let receiver = AddressDataValue::utxo(Discrimination::Test, Value(0));
    let unregistered_account = AddressDataValue::account(Discrimination::Test, Value(100));

    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucet_value(Value(1000))
        .build()
        .expect("cannot build test ledger");

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_all_funds(&mut test_ledger, &unregistered_account, &receiver)
        .get_fragment();

    assert_err!(
        Account(NonExistent),
        test_ledger.apply_transaction(fragment)
    );
}

#[test]
pub fn transaction_with_incorrect_account_spending_counter() {
    let faucet =
        AddressDataValue::account_with_spending_counter(Discrimination::Test, 1, Value(1000));
    let receiver = AddressData::account(Discrimination::Test);

    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucet(&faucet)
        .build()
        .expect("cannot build test ledger");

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_from_faucet(&mut test_ledger, &receiver.into(), Value(1000))
        .get_fragment();
    assert!(
        test_ledger.apply_transaction(fragment).is_err(),
        "first transaction should be successful"
    );
}

#[test]
pub fn repeated_account_transaction() {
    let mut faucet = AddressDataValue::account(Discrimination::Test, Value(200));
    let receiver = AddressDataValue::account(Discrimination::Test, Value(0));

    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucet(&faucet)
        .build()
        .expect("cannot build test ledger");

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_all_funds(&mut test_ledger, &faucet, &receiver)
        .get_fragment();
    assert!(test_ledger.apply_transaction(fragment).is_ok());
    faucet.confirm_transaction().unwrap();
    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_all_funds(&mut test_ledger, &faucet, &receiver)
        .get_fragment();
    assert!(test_ledger.apply_transaction(fragment).is_err());
}
