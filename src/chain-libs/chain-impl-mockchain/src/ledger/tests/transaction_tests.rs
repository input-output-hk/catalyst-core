#![cfg(test)]

use crate::{
    accounting::account::LedgerError::NonExistent,
    date::BlockDate,
    ledger::{
        self,
        check::{TxValidityError, TxVerifyError},
        Error::{Account, InvalidTransactionValidity, TransactionMalformed},
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
    let outputs: Vec<_> = std::iter::repeat(output).take(255).collect();

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_to_outputs_from_faucet(&mut test_ledger, &outputs)
        .get_fragment();

    assert_err!(
        TransactionMalformed(TxVerifyError::TooManyOutputs {
            expected: 254,
            actual: 255
        }),
        test_ledger.apply_transaction(fragment, BlockDate::first())
    );
}

#[test]
pub fn transaction_fail_when_validity_out_of_range() {
    let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new())
        .faucet_value(Value(1000))
        .build()
        .expect("cannot build test ledger");

    let receiver = AddressData::utxo(Discrimination::Test);
    let output = Output {
        address: receiver.address,
        value: Value(1),
    };
    let outputs = [output];

    let valid_until = Some(BlockDate {
        epoch: 10,
        slot_id: 50,
    });

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_to_outputs_from_faucet_with_validity(&mut test_ledger, valid_until, &outputs)
        .get_fragment();

    assert_err!(
        InvalidTransactionValidity(TxValidityError::TransactionExpired),
        test_ledger.apply_transaction(
            fragment,
            BlockDate {
                epoch: 10,
                slot_id: 51,
            }
        )
    );
}

#[test]
pub fn transaction_fail_when_validity_too_far() {
    const MAX_EXPIRY_EPOCHS: u8 = 5;

    let mut test_ledger = LedgerBuilder::from_config(
        ConfigBuilder::new().with_transaction_max_expiry_epochs(MAX_EXPIRY_EPOCHS),
    )
    .faucet_value(Value(1000))
    .build()
    .expect("cannot build test ledger");

    let receiver = AddressData::utxo(Discrimination::Test);
    let output = Output {
        address: receiver.address,
        value: Value(1),
    };

    let valid_until = BlockDate {
        epoch: MAX_EXPIRY_EPOCHS as u32 + 1,
        slot_id: 0,
    };

    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_to_outputs_from_faucet_with_validity(&mut test_ledger, Some(valid_until), &[output])
        .get_fragment();

    assert_err!(
        InvalidTransactionValidity(TxValidityError::TransactionValidForTooLong),
        test_ledger.apply_transaction(fragment, BlockDate::first())
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
    let result = test_ledger.apply_transaction(fragment, BlockDate::first());

    match result {
        Err(err) => panic!("first transaction should be succesful but {}", err),
        Ok(_) => {
            assert_err_match!(
                ledger::Error::Account(crate::account::LedgerError::SpendingCredentialInvalid),
                test_ledger.apply_transaction(fragment2, BlockDate::first())
            );
        }
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
        test_ledger.apply_transaction(fragment, BlockDate::first())
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
        test_ledger
            .apply_transaction(fragment, BlockDate::first())
            .is_err(),
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
    assert!(test_ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_ok());
    faucet.confirm_transaction();
    let fragment = TestTxBuilder::new(test_ledger.block0_hash)
        .move_all_funds(&mut test_ledger, &faucet, &receiver)
        .get_fragment();
    assert!(test_ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_err());
}
