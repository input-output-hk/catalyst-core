use crate::{transaction::WitnessBuilder, Settings};
use chain_addr::Address;
use chain_impl_mockchain::{
    fee::FeeAlgorithm as _,
    transaction::{Input, NoExtra, Output, Transaction, TxBuilderState},
    value::Value,
};

/// Dump all the values into transactions to fill one address
pub struct Dump {
    settings: Settings,
    address: Address,
    inputs: Vec<Input>,
    witness_builders: Vec<WitnessBuilder>,
    outputs: Vec<Transaction<NoExtra>>,
    ignored: Vec<Input>,
}

impl Dump {
    /// dump all the associated input to the given address
    pub fn new(settings: Settings, address: Address) -> Self {
        Self {
            settings,
            address,
            inputs: Vec::with_capacity(255),
            witness_builders: Vec::with_capacity(255),
            outputs: Vec::new(),
            ignored: Vec::new(),
        }
    }

    /// return the list of ignored inputs and the list of built transactions
    ///
    /// the transactions are ready to send
    pub fn finalize(mut self) -> (Vec<Input>, Vec<Transaction<NoExtra>>) {
        self.build_tx_and_clear();

        assert!(
            self.inputs.is_empty(),
            "we should not have any more pending inputs to spend"
        );
        assert!(
            self.witness_builders.is_empty(),
            "we should not have any more pending inputs to spend"
        );

        (self.ignored, self.outputs)
    }

    fn inputs_value(&self) -> Value {
        self.inputs.iter().map(|i| i.value()).sum()
    }

    fn estimate_fee(&self) -> Value {
        self.settings
            .parameters
            .fees
            .calculate(None, self.inputs.len() as u8, 1u8)
    }

    fn mk_output(&self) -> Option<Output<Address>> {
        let fee = self.estimate_fee();
        let input = self.inputs_value();

        let value = input.checked_sub(fee).ok()?;
        let address = self.address.clone();
        Some(Output::from_address(address, value))
    }

    fn build_tx_and_clear(&mut self) {
        if let Some(output) = self.mk_output() {
            let builder = TxBuilderState::new();
            let builder = builder.set_nopayload();
            let builder = builder.set_ios(&self.inputs, &[output]);

            let header_id = self.settings.static_parameters.block0_initial_hash;
            let auth_data = builder.get_auth_data_for_witness().hash();
            let witnesses = std::mem::replace(&mut self.witness_builders, Vec::with_capacity(255));
            let witnesses: Vec<_> = witnesses
                .into_iter()
                .map(move |wb| wb.mk_witness(&header_id, &auth_data))
                .collect();

            let builder = builder.set_witnesses(&witnesses);
            let tx = builder.set_payload_auth(&());
            self.outputs.push(tx);
        } else {
            self.ignored.extend_from_slice(self.inputs.as_slice());
        }

        self.witness_builders.clear();
        self.inputs.clear();
    }

    pub(crate) fn push(&mut self, input: Input, witness_builder: WitnessBuilder) {
        if self.inputs.len() >= 255 {
            self.build_tx_and_clear()
        }

        if self.settings.is_input_worth(&input) {
            self.inputs.push(input);
            self.witness_builders.push(witness_builder);
        } else {
            self.ignored.push(input);
        }
    }
}
