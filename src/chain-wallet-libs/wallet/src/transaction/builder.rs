use crate::{
    transaction::{InputGenerator, WitnessBuilder},
    Settings,
};
use chain_addr::Address;
use chain_impl_mockchain::{
    fee::FeeAlgorithm as _,
    transaction::{
        Input, Output, Payload, SetAuthData, SetIOs, SetWitnesses, Transaction, TxBuilderState,
    },
    value::Value,
};

/// Dump all the values into transactions to fill one address
pub struct TransactionBuilder<P: Payload> {
    settings: Settings,
    outputs: Vec<Output<Address>>,
    inputs: Vec<Input>,
    certificate: P,
    witness_builders: Vec<WitnessBuilder>,
}

// TODO: add more info to the error?
#[derive(Debug, Clone)]
pub struct BalancingError;

impl<P: Payload> TransactionBuilder<P> {
    /// create a new transaction builder with the given settings and outputs
    pub fn new(settings: Settings, outputs: Vec<Output<Address>>, certificate: P) -> Self {
        Self {
            settings,
            outputs,
            certificate,
            inputs: Vec::with_capacity(255),
            witness_builders: Vec::with_capacity(255),
        }
    }

    /// select inputs from the given wallet (InputGenerator)
    ///
    ///
    pub fn select_from<G>(&mut self, input_generator: &mut G) -> bool
    where
        G: InputGenerator,
    {
        let estimated_fee = self.estimate_fee_to_cover_new_input();

        let input_needed = self.outputs_value().saturating_add(estimated_fee);

        if let Some(input) = input_generator.input_to_cover(input_needed) {
            self.inputs.push(input.input);
            self.witness_builders.push(input.witness_builder);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    #[inline]
    pub fn outputs(&self) -> &[Output<Address>] {
        &self.outputs
    }

    #[inline]
    pub fn inputs_value(&self) -> Value {
        self.inputs().iter().map(|i| i.value()).sum()
    }

    #[inline]
    pub fn outputs_value(&self) -> Value {
        self.outputs().iter().map(|i| i.value).sum()
    }

    #[inline]
    fn estimate_fee(&self) -> Value {
        self.settings.parameters.fees.calculate(
            Payload::to_certificate_slice(self.certificate.payload_data().borrow()),
            self.inputs.len() as u8,
            self.outputs.len() as u8,
        )
    }

    #[inline]
    fn estimate_fee_to_cover_new_input(&self) -> Value {
        self.settings.parameters.fees.calculate(
            Payload::to_certificate_slice(self.certificate.payload_data().borrow()),
            self.inputs.len() as u8 + 1u8,
            self.outputs.len() as u8,
        )
    }

    fn check_balance(&self) -> bool {
        let total_in = self.inputs_value();
        let total_out = self.outputs_value();
        let total_fee = self.estimate_fee();

        total_in == total_out.saturating_add(total_fee)
    }

    pub fn finalize_tx(self, auth: <P as Payload>::Auth) -> Result<Transaction<P>, BalancingError> {
        if !self.check_balance() {
            return Err(BalancingError);
        }

        let builder = TxBuilderState::new();
        let builder = builder.set_payload(&self.certificate);

        let builder = self.set_ios(builder);
        let builder = self.set_witnesses(builder);

        Ok(builder.set_payload_auth(&auth))
    }

    fn set_ios(&self, builder: TxBuilderState<SetIOs<P>>) -> TxBuilderState<SetWitnesses<P>> {
        builder.set_ios(&self.inputs, &self.outputs)
    }

    fn set_witnesses(
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

impl std::fmt::Display for BalancingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "transaction building failed, couldn't balance transaction"
        )
    }
}

impl std::error::Error for BalancingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_impl_mockchain::block::Block;
    use chain_impl_mockchain::transaction::NoExtra;
    use chain_ser::deser::Deserialize;
    use hdkeygen::account::Account;

    struct Generator {
        account: Account,
    }

    impl Generator {
        fn new() -> Generator {
            Generator {
                account: Account::from_seed([0u8; 32]),
            }
        }
    }

    impl InputGenerator for Generator {
        fn input_to_cover(&mut self, value: Value) -> Option<crate::transaction::GeneratedInput> {
            let input = Input::from_account_public_key(self.account.account_id().into(), value);
            let witness_builder = WitnessBuilder::Account {
                account: self.account.clone(),
            };
            Some(crate::transaction::GeneratedInput {
                input,
                witness_builder,
            })
        }
    }

    #[test]
    fn build_transaction_with_input_selector() {
        const BLOCK0: &[u8] = include_bytes!("../../../test-vectors/block0");
        let block = Block::deserialize(BLOCK0).unwrap();
        let settings = Settings::new(&block).unwrap();
        let mut builder = TransactionBuilder::new(settings, vec![], NoExtra);

        let mut generator = Generator::new();
        builder.select_from(&mut generator);

        let _tx = builder.finalize_tx(());
    }
}
