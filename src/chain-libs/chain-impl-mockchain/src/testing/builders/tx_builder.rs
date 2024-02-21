use crate::{
    chaintypes::HeaderId,
    date::BlockDate,
    fee::FeeAlgorithm,
    fragment::{Fragment, FragmentId},
    testing::{
        builders::witness_builder::make_witness, data::AddressDataValue, ledger::TestLedger,
        make_witness_with_lane, KeysDb, WitnessMode,
    },
    transaction::{
        Input, NoExtra, Output, OutputsSlice, Transaction, TransactionSignDataHash,
        TransactionSlice, TxBuilder, Witness, WitnessesSlice,
    },
    value::Value,
};
use chain_addr::Address;

pub struct TestTxBuilder {
    block0_hash: HeaderId,
    witness_mode: WitnessMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestTx {
    tx: Transaction<NoExtra>,
}

impl TestTx {
    pub fn new(tx: Transaction<NoExtra>) -> Self {
        TestTx { tx }
    }

    pub fn get_fragment_id(&self) -> FragmentId {
        self.clone().get_fragment().hash()
    }

    pub fn get_fragment(self) -> Fragment {
        Fragment::Transaction(self.tx)
    }

    pub fn hash(&self) -> TransactionSignDataHash {
        self.clone().get_tx().hash()
    }

    pub fn get_tx(self) -> Transaction<NoExtra> {
        self.tx
    }

    pub fn witnesses(&self) -> WitnessesSlice<'_> {
        self.as_slice().witnesses()
    }

    pub fn as_slice(&self) -> TransactionSlice<'_, NoExtra> {
        self.tx.as_slice()
    }

    pub fn get_tx_outputs(&self) -> OutputsSlice<'_> {
        self.as_slice().outputs()
    }
}

impl TestTxBuilder {
    pub fn new(block0_hash: HeaderId) -> Self {
        Self {
            block0_hash,
            witness_mode: Default::default(),
        }
    }

    pub fn witness_mode(mut self, witness_mode: WitnessMode) -> Self {
        self.witness_mode = witness_mode;
        self
    }

    pub fn move_from_faucet(
        &self,
        test_ledger: &mut TestLedger,
        destination: &Address,
        value: Value,
    ) -> TestTx {
        assert_eq!(
            test_ledger.faucets.len(),
            1,
            "method can be used only for single faucet ledger"
        );
        let mut faucet = test_ledger
            .faucets
            .first()
            .cloned()
            .as_mut()
            .expect("test ledger with no faucet configured")
            .clone();
        let fee = test_ledger.fee().fees_for_inputs_outputs(1u8, 1u8);
        let output_value = (value - fee).expect("input value is smaller than fee");
        let inputs = vec![faucet.make_input_with_value(
            test_ledger.find_utxo_for_address(&faucet.clone().into()),
            value,
        )];
        let outputs = vec![Output {
            address: destination.clone(),
            value: output_value,
        }];
        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_expiry_date(BlockDate::first().next_epoch())
            .set_ios(&inputs, &outputs);

        let witness =
            faucet.make_witness(&self.block0_hash, tx_builder.get_auth_data_for_witness());
        let witnesses = vec![witness];

        let tx = tx_builder.set_witnesses(&witnesses).set_payload_auth(&());
        TestTx { tx }
    }

    pub fn move_to_outputs_from_faucet_with_validity(
        &self,
        test_ledger: &mut TestLedger,
        validity: Option<BlockDate>,
        destination: &[Output<Address>],
    ) -> TestTx {
        assert_eq!(
            test_ledger.faucets.len(),
            1,
            "method can be used only for single faucet ledger"
        );
        let mut faucet = test_ledger
            .faucets
            .first()
            .as_mut()
            .expect("test ledger with no faucet configured")
            .clone();
        let input_val = Value::sum(destination.iter().map(|o| o.value)).unwrap();
        let inputs = vec![faucet.make_input_with_value(
            test_ledger.find_utxo_for_address(&faucet.clone().into()),
            input_val,
        )];
        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_expiry_date(validity.unwrap_or_else(BlockDate::first))
            .set_ios(&inputs, destination);

        let witness =
            faucet.make_witness(&self.block0_hash, tx_builder.get_auth_data_for_witness());
        let witnesses = vec![witness];

        let tx = tx_builder.set_witnesses(&witnesses).set_payload_auth(&());
        TestTx { tx }
    }

    pub fn move_to_outputs_from_faucet(
        &self,
        test_ledger: &mut TestLedger,
        destination: &[Output<Address>],
    ) -> TestTx {
        self.move_to_outputs_from_faucet_with_validity(test_ledger, None, destination)
    }

    pub fn move_all_funds(
        &self,
        test_ledger: &mut TestLedger,
        source: &AddressDataValue,
        destination: &AddressDataValue,
    ) -> TestTx {
        let mut keys_db = KeysDb::empty();
        keys_db.add_key(source.private_key());
        keys_db.add_key(destination.private_key());
        self.move_funds(test_ledger, source, destination, source.value)
    }

    pub fn move_funds(
        &self,
        test_ledger: &mut TestLedger,
        source: &AddressDataValue,
        destination: &AddressDataValue,
        value: Value,
    ) -> TestTx {
        let fee = test_ledger.fee();
        let fee_value = (fee.fees_for_inputs_outputs(1u8, 1u8) + Value(fee.constant)).unwrap();
        let output_value = (value - fee_value).expect("input value is smaller than fee");
        let sources = vec![AddressDataValue::new(source.address_data(), value)];
        let destinations = vec![AddressDataValue::new(
            destination.address_data(),
            output_value,
        )];
        self.move_funds_multiple(test_ledger, &sources, &destinations)
    }

    pub fn move_funds_multiple(
        &self,
        test_ledger: &mut TestLedger,
        sources: &[AddressDataValue],
        destinations: &[AddressDataValue],
    ) -> TestTx {
        let inputs: Vec<Input> = sources
            .iter()
            .cloned()
            .map(|x| {
                let optional_utxo = test_ledger.find_utxo_for_address(&x.address_data());
                x.make_input(optional_utxo)
            })
            .collect();
        let destinations: Vec<Output<Address>> = destinations
            .iter()
            .cloned()
            .map(|x| x.make_output())
            .collect();
        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_expiry_date(BlockDate::first().next_epoch())
            .set_ios(&inputs, &destinations);

        let witnesses: Vec<Witness> = {
            if matches!(self.witness_mode, WitnessMode::None) {
                vec![]
            } else {
                sources
                    .iter()
                    .map(|source| {
                        let auth_data_hash = tx_builder.get_auth_data_for_witness().hash();

                        match self.witness_mode {
                            WitnessMode::None => unreachable!(),
                            WitnessMode::Default => make_witness(
                                &self.block0_hash,
                                &source.address_data(),
                                &auth_data_hash,
                            ),
                            WitnessMode::Account { lane } => make_witness_with_lane(
                                &self.block0_hash,
                                &source.address_data(),
                                lane,
                                &auth_data_hash,
                            ),
                        }
                    })
                    .collect()
            }
        };

        let tx = tx_builder.set_witnesses(&witnesses).set_payload_auth(&());
        TestTx { tx }
    }
}
