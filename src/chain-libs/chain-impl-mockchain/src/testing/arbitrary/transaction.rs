use super::ArbitraryAddressDataVec;
use crate::{
    account::Ledger as AccountLedger,
    fee::LinearFee,
    testing::{
        arbitrary::{utils as arbitrary_utils, AverageValue},
        data::{AddressData, AddressDataValue},
        ledger::TestLedger,
    },
    transaction::{Input, Output},
    value::*,
};
use chain_addr::{Address, Kind};
use chain_crypto::{Ed25519, PublicKey};
use quickcheck::{Arbitrary, Gen};
use std::iter;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct ArbitraryValidTransactionData {
    pub addresses: Vec<AddressDataValue>,
    pub input_addresses: Vec<AddressDataValue>,
    pub output_addresses: Vec<AddressDataValue>,
    pub fee: LinearFee,
}

impl Arbitrary for ArbitraryValidTransactionData {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        use ArbitraryValidTransactionData as tx_data;
        let source = ArbitraryAddressDataVec::arbitrary(gen);
        let values: Vec<Value> = iter::from_fn(|| Some(AverageValue::arbitrary(gen)))
            .map(|x| x.into())
            .take(source.0.len())
            .collect();
        let addresses_values: Vec<AddressDataValue> =
            tx_data::zip_addresses_and_values(&source.0, values);
        let input_addresses_values =
            arbitrary_utils::choose_random_vec_subset(&addresses_values, gen);
        let total_input_value = input_addresses_values
            .iter()
            .cloned()
            .map(|x| x.value.0)
            .sum();
        let (output_addresses_values, fee) =
            tx_data::choose_random_output_subset(&addresses_values, total_input_value, gen);

        ArbitraryValidTransactionData::new(
            addresses_values,
            input_addresses_values,
            output_addresses_values,
            fee,
        )
    }
}

impl ArbitraryValidTransactionData {
    pub fn new(
        addresses: Vec<AddressDataValue>,
        input_addresses_values: Vec<AddressDataValue>,
        output_addresses_values: Vec<AddressDataValue>,
        fee: LinearFee,
    ) -> Self {
        ArbitraryValidTransactionData {
            addresses,
            input_addresses: input_addresses_values,
            output_addresses: output_addresses_values,
            fee,
        }
    }

    fn zip_addresses_and_values(
        addresses: &[AddressData],
        values: Vec<Value>,
    ) -> Vec<AddressDataValue> {
        addresses
            .iter()
            .cloned()
            .zip(values.iter())
            .map(|(x, y)| AddressDataValue::new(x, *y))
            .collect()
    }

    fn choose_random_output_subset<G: Gen>(
        source: &[AddressDataValue],
        total_input_funds: u64,
        gen: &mut G,
    ) -> (Vec<AddressDataValue>, LinearFee) {
        let mut outputs: Vec<AddressData> = Vec::new();
        let mut funds_per_output: u64 = 0;

        // keep choosing random subset from source until each output will recieve at least 1 coin
        // since zero output is not allowed
        // TODO: randomize funds per output
        while funds_per_output == 0 {
            outputs = arbitrary_utils::choose_random_vec_subset(source, gen)
                .iter()
                .cloned()
                .map(|x| x.address_data)
                .collect();
            funds_per_output = total_input_funds / outputs.len() as u64;
        }

        let output_address_len = outputs.len() as u64;
        let remainder = total_input_funds - (output_address_len * funds_per_output);
        let fee = LinearFee::new(remainder, 0, 0);
        (
            Self::distribute_values_for_outputs(outputs, funds_per_output),
            fee,
        )
    }

    fn distribute_values_for_outputs(
        outputs: Vec<AddressData>,
        funds_per_output: u64,
    ) -> Vec<AddressDataValue> {
        outputs
            .iter()
            .cloned()
            .zip(iter::from_fn(|| Some(Value(funds_per_output))))
            .map(|(x, y)| AddressDataValue::new(x, y))
            .collect()
    }

    fn make_single_input(
        &self,
        address_data_value: AddressDataValue,
        ledger: &TestLedger,
    ) -> Input {
        let utxo_option = ledger.find_utxo_for_address(&address_data_value.address_data);
        address_data_value.make_input(utxo_option)
    }

    pub fn make_inputs(&mut self, ledger: &TestLedger) -> Vec<Input> {
        self.input_addresses
            .iter()
            .cloned()
            .map(|x| self.make_single_input(x, ledger))
            .collect()
    }

    pub fn make_outputs_from_all_addresses(&self) -> Vec<Output<Address>> {
        self.addresses.iter().map(|x| x.make_output()).collect()
    }

    pub fn make_outputs(&mut self) -> Vec<Output<Address>> {
        self.output_addresses
            .iter()
            .map(|x| x.make_output())
            .collect()
    }

    pub fn input_addresses(&mut self) -> Vec<AddressData> {
        self.input_addresses
            .iter()
            .cloned()
            .map(|x| x.address_data)
            .collect()
    }
}

pub struct AccountStatesVerifier(pub ArbitraryValidTransactionData);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Cannot find coresponding account address for expected {element}")]
    AccountNotFound { element: PublicKey<Ed25519> },
    #[error("Cannot find expected output: {output} with value: {value}")]
    UtxoNotFound {
        output: PublicKey<Ed25519>,
        value: Value,
    },
    #[error(
        "Address funds are different for {element} than expected: {expected}, but got: {actual}"
    )]
    WrongValue {
        element: PublicKey<Ed25519>,
        expected: Value,
        actual: Value,
    },
}

impl AccountStatesVerifier {
    pub fn new(transaction_data: ArbitraryValidTransactionData) -> Self {
        AccountStatesVerifier(transaction_data)
    }

    pub fn calculate_current_account_states(&self) -> Vec<AddressDataValue> {
        let inputs = &self.0.input_addresses;
        let snapshot: Vec<AddressDataValue> = self
            .0
            .addresses
            .iter()
            .filter(|&x| filter_accounts(x))
            .cloned()
            .map(|x| find_equal_and_sub(x, inputs))
            .collect();

        let outputs = &self.0.output_addresses;
        let snapshot: Vec<AddressDataValue> = snapshot
            .iter()
            .cloned()
            .map(|x| find_equal_and_add(x, outputs))
            .collect();
        snapshot
    }

    pub fn verify(&self, accounts: &AccountLedger) -> Result<(), Error> {
        let account_snapshot = self.calculate_current_account_states();
        for address_data_value in account_snapshot.iter() {
            let result = accounts.get_state(&address_data_value.address_data.public_key().into());
            match result {
                Ok(state) => {
                    if state.value != address_data_value.value {
                        return Err(Error::WrongValue {
                            element: address_data_value.address_data.public_key(),
                            actual: state.value,
                            expected: address_data_value.value,
                        });
                    }
                }
                Err(_) => {
                    return Err(Error::AccountNotFound {
                        element: address_data_value.address_data.public_key(),
                    })
                }
            }
        }
        Ok(())
    }
}

fn find_equal_and_sub(x: AddressDataValue, collection: &[AddressDataValue]) -> AddressDataValue {
    match collection
        .iter()
        .find(|&y| y.address_data == x.address_data)
        .cloned()
    {
        Some(y) => AddressDataValue::new(x.address_data, (x.value - y.value).unwrap()),
        None => x,
    }
}

fn find_equal_and_add(x: AddressDataValue, collection: &[AddressDataValue]) -> AddressDataValue {
    match collection
        .iter()
        .find(|&y| y.address_data == x.address_data)
        .cloned()
    {
        Some(y) => AddressDataValue::new(x.address_data, (x.value + y.value).unwrap()),
        None => x,
    }
}

fn filter_accounts(x: &AddressDataValue) -> bool {
    matches!(x.address_data.kind(), Kind::Account { .. })
}

fn filter_utxo(x: &AddressDataValue) -> bool {
    matches!(
        x.address_data.kind(),
        Kind::Single { .. } | Kind::Group { .. }
    )
}

pub struct UtxoVerifier(pub ArbitraryValidTransactionData);

impl UtxoVerifier {
    pub fn new(transaction_data: ArbitraryValidTransactionData) -> Self {
        UtxoVerifier(transaction_data)
    }
    #[allow(clippy::iter_overeager_cloned)]
    pub fn calculate_current_utxo(&self) -> Vec<AddressDataValue> {
        let inputs = &self.0.input_addresses;
        let all = &self.0.addresses;
        let outputs = &self.0.output_addresses;

        let utxo_not_changed: Vec<AddressDataValue> = all
            .iter()
            .filter(|&x| filter_utxo(x))
            .cloned()
            .filter(|x| !inputs.contains(x))
            .collect();
        let utxo_added: Vec<AddressDataValue> = outputs
            .iter()
            .filter(|&x| filter_utxo(x))
            .cloned()
            .collect();

        let mut snapshot = Vec::new();
        snapshot.extend(utxo_not_changed);
        snapshot.extend(utxo_added);
        snapshot
    }

    pub fn verify(&self, ledger: &TestLedger) -> Result<(), Error> {
        let expected_utxo_snapshots = &self.calculate_current_utxo();
        for utxo_snapshot in expected_utxo_snapshots {
            let condition = !ledger.utxos().any(|x| {
                x.output.address.clone() == utxo_snapshot.address_data.address.clone()
                    && x.output.value.0 == utxo_snapshot.value.0
            });
            if condition {
                return Err(Error::UtxoNotFound {
                    output: utxo_snapshot.address_data.public_key(),
                    value: utxo_snapshot.value,
                });
            }
        }
        Ok(())
    }
}
