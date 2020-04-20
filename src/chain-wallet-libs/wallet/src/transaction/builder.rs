use crate::{transaction::WitnessBuilder, Settings};
use chain_addr::Address;
use chain_impl_mockchain::{
    fee::FeeAlgorithm as _,
    transaction::{
        Input, NoExtra, Output, Payload, SetAuthData, SetIOs, SetWitnesses, Transaction,
        TxBuilderState,
    },
    value::Value,
};

/// Dump all the values into transactions to fill one address
pub struct TransactionBuilder {
    settings: Settings,
    outputs: Vec<Output<Address>>,
    inputs: Vec<Input>,
    witness_builders: Vec<WitnessBuilder>,
}

impl TransactionBuilder {
    /// dump all the associated input to the given address
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            outputs: Vec::new(),
            inputs: Vec::with_capacity(255),
            witness_builders: Vec::with_capacity(255),
        }
    }

    pub fn inputs_value(&self) -> Value {
        self.inputs.iter().map(|i| i.value()).sum()
    }

    pub fn outputs_value(&self) -> Value {
        self.outputs.iter().map(|i| i.value).sum()
    }

    fn estimate_fee(&self) -> Value {
        self.settings.parameters.fees.calculate(
            None,
            self.inputs.len() as u8,
            self.outputs.len() as u8,
        )
    }

    fn check_balance(&self) -> bool {
        let total_in = self.inputs_value();
        let total_out = self.outputs_value();
        let total_fee = self.estimate_fee();

        total_in == total_out.saturating_add(total_fee)
    }

    pub fn finalize_tx(self) -> Transaction<NoExtra> {
        if !self.check_balance() {
            todo!()
        }

        let builder = TxBuilderState::new();
        let builder = builder.set_nopayload();

        let builder = self.set_ios(builder);
        let builder = self.set_witnesses(builder);

        builder.set_payload_auth(&())
    }

    fn set_ios<P>(&self, builder: TxBuilderState<SetIOs<P>>) -> TxBuilderState<SetWitnesses<P>> {
        builder.set_ios(&self.inputs, &self.outputs)
    }

    fn set_witnesses<P>(
        &self,
        builder: TxBuilderState<SetWitnesses<P>>,
    ) -> TxBuilderState<SetAuthData<P>>
    where
        P: Payload,
    {
        let header_id = self.settings.static_parameters.block0_initial_hash;
        let auth_data = builder.get_auth_data_for_witness().hash();
        let witnesses: Vec<_> = self
            .witness_builders
            .iter()
            .map(move |wb| wb.mk_witness(&header_id, &auth_data))
            .collect();

        builder.set_witnesses(&witnesses)
    }
}
