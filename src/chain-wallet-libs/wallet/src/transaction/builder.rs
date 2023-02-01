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
pub enum Error {
    #[error("Cannot balance the transaction")]
    BalancingError,
    #[error("Invalid witness input amount, expected: {0}, provided: {1}")]
    InvalidWitnessInputAmount(usize, usize),
}

pub struct TransactionBuilder<P: Payload, SecretKey, WitnessData, Signature> {
    settings: Settings,
    payload: P,
    validity: BlockDate,
    outputs: Vec<Output<Address>>,
    inputs: Vec<Input>,
    witness_builders: Vec<Box<dyn WitnessBuilder<SecretKey, WitnessData, Signature>>>,
}

pub enum AddInputStatus {
    Added,
    Skipped(Input),
    NotEnoughSpace,
}

#[derive(Clone)]
pub enum WitnessInput<SecretKey, Signature> {
    SecretKey(SecretKey),
    Signature(Signature),
}

impl<P: Payload, SecretKey, WitnessData: AsRef<[u8]>, Signature>
    TransactionBuilder<P, SecretKey, WitnessData, Signature>
{
    /// create a new transaction builder with the given settings and outputs
    pub fn new(settings: Settings, payload: P, validity: BlockDate) -> Self {
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

    pub fn add_input_if_worth<B: WitnessBuilder<SecretKey, WitnessData, Signature> + 'static>(
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

    pub fn add_input<B: WitnessBuilder<SecretKey, WitnessData, Signature> + 'static>(
        &mut self,
        input: Input,
        witness_builder: B,
    ) -> bool {
        if self.inputs().len() < 255 {
            self.inputs.push(input);
            self.witness_builders.push(Box::new(witness_builder));
            true
        } else {
            false
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

    fn prepare_tx(&self) -> Result<TxBuilderState<SetWitnesses<P>>, Error> {
        if !matches!(self.check_balance(), Balance::Zero) {
            return Err(Error::BalancingError);
        }

        let builder = TxBuilderState::new();
        let builder = builder.set_payload(&self.payload);
        let builder = self.set_validity(builder);
        let builder = self.set_ios(builder);
        Ok(builder)
    }

    pub fn get_sign_data(&self) -> Result<Vec<WitnessData>, Error> {
        let builder = self.prepare_tx()?;
        let header_id = self.settings.block0_initial_hash;
        let auth_data = builder.get_auth_data_for_witness().hash();
        Ok(self
            .witness_builders
            .iter()
            .map(|witness_builder| witness_builder.build_sign_data(&header_id, &auth_data))
            .collect())
    }

    pub fn finalize_tx(
        self,
        auth: <P as Payload>::Auth,
        witness_input: Vec<WitnessInput<SecretKey, Signature>>,
    ) -> Result<Transaction<P>, Error> {
        let builder = self.prepare_tx()?;
        let builder = self.set_witnesses(builder, witness_input)?;
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
        witness_input: Vec<WitnessInput<SecretKey, Signature>>,
    ) -> Result<TxBuilderState<SetAuthData<P>>, Error> {
        let header_id = self.settings.block0_initial_hash;
        let auth_data = builder.get_auth_data_for_witness().hash();

        if witness_input.len() != self.witness_builders.len() {
            return Err(Error::InvalidWitnessInputAmount(
                self.witness_builders.len(),
                witness_input.len(),
            ));
        }
        let mut witnesses = Vec::new();
        witness_input
            .into_iter()
            .enumerate()
            .for_each(|(i, witness_input)| {
                let witness_builder = &self.witness_builders[i];
                let witness = match witness_input {
                    WitnessInput::SecretKey(secret_key) => witness_builder.sign(
                        witness_builder.build_sign_data(&header_id, &auth_data),
                        secret_key,
                    ),
                    WitnessInput::Signature(signature) => witness_builder.build(signature),
                };
                witnesses.push(witness);
            });

        Ok(builder.set_witnesses(&witnesses))
    }
}
