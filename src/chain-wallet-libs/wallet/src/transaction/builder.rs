use crate::{
    store::UtxoStore,
    transaction::{InputStrategy, OutputStrategy, Strategy, WitnessBuilder},
    Settings,
};
use chain_addr::Address;
use chain_impl_mockchain::{
    fee::FeeAlgorithm as _,
    transaction::{
        Balance, Input, Output, Payload, SetAuthData, SetIOs, SetWitnesses, Transaction,
        TxBuilderState,
    },
    value::Value,
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Cannot balance the transaction")]
pub struct BalancingError;

pub struct TransactionBuilder<P: Payload> {
    settings: Settings,
    strategy: Strategy,
    payload: P,
    outputs: Vec<Output<Address>>,
    inputs: Vec<Input>,
    witness_builders: Vec<WitnessBuilder>,
}

impl<P: Payload> TransactionBuilder<P> {
    /// create a new transaction builder with the given settings and outputs
    pub fn new(
        settings: Settings,
        strategy: Strategy,
        outputs: Vec<Output<Address>>,
        payload: P,
    ) -> Self {
        Self {
            settings,
            strategy,
            outputs,
            payload,
            inputs: Vec::with_capacity(255),
            witness_builders: Vec::with_capacity(255),
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
    pub fn estimate_fee_with(&self, extra_inputs: u8, extra_outputs: u8) -> Value {
        self.settings.parameters.fees.calculate(
            Payload::to_certificate_slice(self.payload.payload_data().borrow()),
            self.inputs.len() as u8 + extra_inputs,
            self.outputs.len() as u8 + extra_outputs,
        )
    }

    #[inline]
    pub fn estimate_fee(&self) -> Value {
        self.estimate_fee_with(0, 0)
    }

    pub fn populate_with_utxos<K>(&mut self, utxo_store: &UtxoStore<K>) {
        match self.strategy.input() {
            InputStrategy::BestEffort => {
                // in case of a best effort, the utxos are selected in by increasing order
                // the goal being to take us as closely as possible to the target value
                // by adding values one by one in increasing order
                let mut utxos = utxo_store.utxos();

                while let Some(utxo) = utxos.next() {
                    match self.check_balance_with(1, 0) {
                        Balance::Zero => break,
                        Balance::Positive(_) => break,
                        Balance::Negative(missing) => {
                            if utxo.value <= missing {
                                let input = Input::from_utxo(*utxo.as_ref());
                                self.inputs.push(input);
                            }
                        }
                    }
                    //
                }
            }
            InputStrategy::PrivacyPreserving => {
                utxo_store.groups();
            }
        }
    }

    pub fn check_balance(&self) -> Balance {
        self.check_balance_with(0, 0)
    }

    pub fn check_balance_with(&self, extra_inputs: u8, extra_outputs: u8) -> Balance {
        let total_in = self.inputs_value();
        let total_out = self.outputs_value();
        let total_fee = self.estimate_fee_with(extra_inputs, extra_outputs);

        let total_out = total_out.saturating_add(total_fee);

        match total_in.cmp(&total_out) {
            std::cmp::Ordering::Greater => {
                Balance::Positive(total_in.checked_sub(total_out).unwrap())
            }
            std::cmp::Ordering::Equal => Balance::Zero,
            std::cmp::Ordering::Less => Balance::Negative(total_out.checked_sub(total_in).unwrap()),
        }
    }

    pub fn finalize_tx(self, auth: <P as Payload>::Auth) -> Result<Transaction<P>, BalancingError> {
        if !matches!(self.check_balance(), Balance::Zero) {
            return Err(BalancingError);
        }

        let builder = TxBuilderState::new();
        let builder = builder.set_payload(&self.payload);

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
