use super::witness_builder::WitnessBuilder;
use crate::Settings;
use chain_addr::Address;
use chain_impl_mockchain::{
    block::BlockDate,
    fee::FeeAlgorithm as _,
    transaction::{
        Balance, Input, Output, Payload, SetAuthData, SetIOs, SetTtl, SetWitnesses, Transaction,
        TxBuilderState,
    },
    value::Value,
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Cannot balance the transaction")]
pub struct BalancingError;

pub struct TransactionBuilder<'settings, P: Payload> {
    settings: &'settings Settings,
    payload: P,
    validity: BlockDate,
    outputs: Vec<Output<Address>>,
    inputs: Vec<Input>,
    witness_builders: Vec<Box<dyn WitnessBuilder>>,
}

pub enum AddInputStatus {
    Added,
    Skipped(Input),
    NotEnoughSpace,
}

impl<'settings, P: Payload> TransactionBuilder<'settings, P> {
    /// create a new transaction builder with the given settings and outputs
    pub fn new(settings: &'settings Settings, payload: P, validity: BlockDate) -> Self {
        Self {
            settings,
            payload,
            validity,
            outputs: Vec::with_capacity(255),
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
        self.settings.fees.calculate(
            self.payload
                .payload_data()
                .borrow()
                .into_certificate_slice(),
            self.inputs.len() as u8 + extra_inputs,
            self.outputs.len() as u8 + extra_outputs,
        )
    }

    #[inline]
    pub fn estimate_fee(&self) -> Value {
        self.estimate_fee_with(0, 0)
    }

    pub fn add_input_if_worth<B: WitnessBuilder + 'static>(
        &mut self,
        input: Input,
        witness_builder: B,
    ) -> AddInputStatus {
        if self.settings.is_input_worth(&input) {
            match self.add_input(input, witness_builder) {
                true => AddInputStatus::Added,
                false => AddInputStatus::NotEnoughSpace,
            }
        } else {
            AddInputStatus::Skipped(input)
        }
    }

    pub fn add_input<B: WitnessBuilder + 'static>(
        &mut self,
        input: Input,
        witness_builder: B,
    ) -> bool {
        match self.inputs.len().cmp(&255) {
            std::cmp::Ordering::Less => {
                self.inputs.push(input);
                self.witness_builders.push(Box::new(witness_builder));
                true
            }
            _ => false,
        }
    }

    pub fn add_output(&mut self, output: Output<Address>) -> bool {
        if self.outputs().len() < 255 {
            self.outputs.push(output);
            true
        } else {
            false
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

        let builder = self.set_validity(builder);
        let builder = self.set_ios(builder);
        let builder = self.set_witnesses(builder);

        Ok(builder.set_payload_auth(&auth))
    }

    fn set_validity(&self, builder: TxBuilderState<SetTtl<P>>) -> TxBuilderState<SetIOs<P>> {
        builder.set_expiry_date(self.validity)
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
        let header_id = self.settings.block0_initial_hash;
        let auth_data = builder.get_auth_data_for_witness().hash();
        let witnesses: Vec<_> = self
            .witness_builders
            .iter()
            .map(|wb| wb.build(&header_id, &auth_data))
            .collect();

        builder.set_witnesses(&witnesses)
    }
}
