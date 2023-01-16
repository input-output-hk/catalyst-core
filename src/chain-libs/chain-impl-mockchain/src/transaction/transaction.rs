use super::builder::{SetAuthData, TxBuilder, TxBuilderState};
use super::element::{Balance, BalanceError, TransactionSignDataHash};
use super::input::{Input, INPUT_SIZE};
use super::payload::{Payload, PayloadAuthSlice, PayloadSlice};
use super::transfer::Output;
use super::witness::Witness;
use crate::date::BlockDate;
use crate::value::{Value, ValueError};
use chain_addr::Address;
use chain_core::{
    packer::Codec,
    property::{Deserialize, DeserializeFromSlice},
};
use chain_crypto::digest::Digest;
use std::fmt::{self, Debug};
use std::marker::PhantomData;

pub struct Transaction<P> {
    pub(super) data: Box<[u8]>,
    pub(super) tstruct: TransactionStruct,
    pub(super) phantom: PhantomData<P>,
}

impl<P> Clone for Transaction<P> {
    fn clone(&self) -> Self {
        Transaction {
            data: self.data.clone(),
            tstruct: self.tstruct.clone(),
            phantom: self.phantom,
        }
    }
}

impl<P> Debug for Transaction<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tx = self.as_slice();
        f.debug_struct("Transaction")
            .field("payload", &tx.payload().0)
            .field("nb_inputs", &tx.nb_inputs())
            .field("nb_outputs", &tx.nb_outputs())
            .field("valid_until", &tx.valid_until())
            .field("nb_witnesses", &tx.nb_witnesses())
            .field("total_input_value", &self.total_input())
            .field("total_output_value", &self.total_output())
            .finish()
    }
}

impl<P> PartialEq for Transaction<P> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl<P> Eq for Transaction<P> {}

pub struct TransactionSlice<'a, P: ?Sized> {
    pub(super) data: &'a [u8],
    pub(super) tstruct: TransactionStruct,
    pub(super) phantom: PhantomData<P>,
}

pub struct UnverifiedTransactionSlice<'a, P: ?Sized> {
    data: &'a [u8],
    phantom: PhantomData<P>,
}

#[derive(Clone)]
pub struct TransactionAuthData<'a>(pub &'a [u8]);

impl<'a> TransactionAuthData<'a> {
    pub fn hash(&self) -> TransactionSignDataHash {
        Digest::digest(self.0).into()
    }
}

#[derive(Clone)]
pub struct TransactionBindingAuthData<'a>(pub &'a [u8]);

#[derive(Clone)]
pub struct InputsSlice<'a>(u8, &'a [u8]);

#[derive(Clone)]
pub struct OutputsSlice<'a>(u8, &'a [u8]);

#[derive(Clone)]
pub struct WitnessesSlice<'a>(u8, &'a [u8]);

#[derive(Clone)]
pub struct InputsWitnessesSlice<'a>(InputsSlice<'a>, WitnessesSlice<'a>);

pub struct InputsIter<'a> {
    index: usize, // in number of inputs
    slice: InputsSlice<'a>,
}

pub struct OutputsIter<'a> {
    index: usize, // in bytes
    slice: OutputsSlice<'a>,
}

pub struct WitnessesIter<'a> {
    index: usize, // in bytes
    slice: WitnessesSlice<'a>,
}

pub struct InputsWitnessesIter<'a> {
    iiter: InputsIter<'a>,
    witer: WitnessesIter<'a>,
}

impl<'a> Iterator for InputsIter<'a> {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= (self.slice.0 as usize) {
            None
        } else {
            let offset = self.index * INPUT_SIZE;
            let mut input = [0u8; INPUT_SIZE];
            input.copy_from_slice(&self.slice.1[offset..offset + INPUT_SIZE]);
            self.index += 1;
            Some(input.into())
        }
    }
}

impl<'a> Iterator for OutputsIter<'a> {
    type Item = Output<Address>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.slice.1.len() {
            None
        } else {
            let mut codec = Codec::new(self.slice.1);
            codec.skip_bytes(self.index);
            let output = Output::deserialize_from_slice(&mut codec).unwrap();
            self.index = self.slice.1.len() - codec.bytes_left();
            Some(output)
        }
    }
}

impl<'a> Iterator for WitnessesIter<'a> {
    type Item = Witness;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.slice.1.len() {
            None
        } else {
            let mut codec = Codec::new(self.slice.1);
            codec.skip_bytes(self.index);
            let output = Witness::deserialize_from_slice(&mut codec).unwrap();
            self.index = self.slice.1.len() - codec.bytes_left();
            Some(output)
        }
    }
}

impl<'a> Iterator for InputsWitnessesIter<'a> {
    type Item = (Input, Witness);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.iiter.next(), self.witer.next()) {
            (None, None) => None,
            (Some(i), Some(w)) => Some((i, w)),
            (None, _) => {
                panic!("internal error during inputs-witnesses iter: inputs are short of witnesses")
            }
            (_, None) => {
                panic!("internal error during inputs-witnesses iter: witnesses are short of inputs")
            }
        }
    }
}

impl<'a> InputsSlice<'a> {
    pub fn nb_inputs(&self) -> u8 {
        self.0
    }

    pub fn iter(self) -> InputsIter<'a> {
        InputsIter {
            index: 0,
            slice: self,
        }
    }
}

impl<'a> OutputsSlice<'a> {
    pub fn nb_outputs(&self) -> u8 {
        self.0
    }

    pub fn iter(self) -> OutputsIter<'a> {
        OutputsIter {
            index: 0,
            slice: self,
        }
    }
}

impl<'a> WitnessesSlice<'a> {
    pub fn nb_witnesses(&self) -> u8 {
        self.0
    }

    pub fn iter(self) -> WitnessesIter<'a> {
        WitnessesIter {
            index: 0,
            slice: self,
        }
    }
}

impl<'a> InputsWitnessesSlice<'a> {
    pub fn nb_elements(&self) -> u8 {
        (self.0).0
    }

    pub fn iter(self) -> InputsWitnessesIter<'a> {
        InputsWitnessesIter {
            iiter: self.0.iter(),
            witer: self.1.iter(),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TransactionStructError {
    #[error("number of inputs is invalid")]
    CannotReadNbInputs,
    #[error("number of outputs is invalid")]
    CannotReadNbOutputs,
    #[error("expiry block date is invalid")]
    CannotReadExpiryDate,
    #[error("transaction payload is invalid")]
    PayloadInvalid,
    #[error("invalid inputs")]
    InputsInvalid,
    #[error("invalid outputs")]
    OutputsInvalid,
    #[error("invalid witnesses")]
    WitnessesInvalid,
    #[error("spurious trailing data")]
    SpuriousTrailingData,
    #[error("payload authentication is missing")]
    PayloadAuthMissing,
    #[error("payload authentication is invalid")]
    PayloadAuthInvalid,
}

#[derive(Clone)]
pub(super) struct TransactionStruct {
    pub(super) sz: usize,
    pub(super) nb_inputs: u8,
    pub(super) nb_outputs: u8,
    // TODO: remove this
    pub(super) valid_until: BlockDate,
    pub(super) inputs: usize,
    pub(super) outputs: usize,
    pub(super) witnesses: usize,
    pub(super) payload_auth: usize,
}

/// Verify the structure of the transaction and return all the offsets
fn get_spine<P: Payload>(slice: &[u8]) -> Result<TransactionStruct, TransactionStructError> {
    let sz = slice.len();
    let mut codec = Codec::new(slice);

    // read payload
    if P::HAS_DATA {
        P::deserialize_validate_from_slice(&mut codec)
            .map_err(|_| TransactionStructError::PayloadInvalid)?;
    }

    // read date
    let epoch = codec
        .get_be_u32()
        .map_err(|_| TransactionStructError::CannotReadExpiryDate)?;
    let slot_id = codec
        .get_be_u32()
        .map_err(|_| TransactionStructError::CannotReadExpiryDate)?;
    let valid_until = BlockDate { epoch, slot_id };

    // read input and outputs
    let nb_inputs = codec
        .get_u8()
        .map_err(|_| TransactionStructError::CannotReadNbInputs)?;
    let nb_outputs = codec
        .get_u8()
        .map_err(|_| TransactionStructError::CannotReadNbOutputs)?;

    let inputs_pos = sz - codec.bytes_left();
    codec.skip_bytes(nb_inputs as usize * INPUT_SIZE);
    let outputs_pos = sz - codec.bytes_left();
    for _ in 0..nb_outputs {
        Output::<Address>::deserialize_validate(&mut codec)
            .map_err(|_| TransactionStructError::OutputsInvalid)?;
    }

    // read witnesses
    let witnesses_pos = sz - codec.bytes_left();
    for _ in 0..nb_inputs {
        Witness::deserialize_validate_from_slice(&mut codec)
            .map_err(|_| TransactionStructError::WitnessesInvalid)?;
    }

    // read payload auth
    let payload_auth_pos = sz - codec.bytes_left();
    if P::HAS_DATA && P::HAS_AUTH {
        if !codec.has_bytes_left() {
            return Err(TransactionStructError::PayloadAuthMissing);
        }
        P::Auth::deserialize_validate_from_slice(&mut codec)
            .map_err(|_| TransactionStructError::PayloadAuthInvalid)?;
    }

    if codec.has_bytes_left() {
        return Err(TransactionStructError::SpuriousTrailingData);
    }
    Ok(TransactionStruct {
        sz,
        nb_inputs,
        nb_outputs,
        valid_until,
        inputs: inputs_pos,
        outputs: outputs_pos,
        witnesses: witnesses_pos,
        payload_auth: payload_auth_pos,
    })
}

impl<'a, P: Payload> From<&'a [u8]> for UnverifiedTransactionSlice<'a, P> {
    fn from(slice: &'a [u8]) -> Self {
        UnverifiedTransactionSlice {
            data: slice,
            phantom: PhantomData,
        }
    }
}

impl<'a, P: Payload> UnverifiedTransactionSlice<'a, P> {
    pub fn check(self) -> Result<TransactionSlice<'a, P>, TransactionStructError> {
        let tstruct = get_spine::<P>(self.data)?;
        Ok(TransactionSlice {
            data: self.data,
            tstruct,
            phantom: self.phantom,
        })
    }
}

impl<P> Transaction<P> {
    pub fn as_slice(&self) -> TransactionSlice<'_, P> {
        TransactionSlice {
            data: &self.data,
            tstruct: self.tstruct.clone(),
            phantom: self.phantom,
        }
    }

    pub fn hash(&self) -> TransactionSignDataHash {
        self.as_slice().transaction_sign_data_hash()
    }

    pub fn nb_inputs(&self) -> u8 {
        self.tstruct.nb_inputs
    }

    /// Create a specific block0 payload transaction
    pub fn block0_payload(payload: &P, payload_auth: &P::Auth) -> Transaction<P>
    where
        P: Payload,
    {
        TxBuilder::new()
            .set_payload(payload)
            .set_expiry_date(BlockDate::first().next_epoch())
            .set_ios(&[], &[])
            .set_witnesses(&[])
            .set_payload_auth(payload_auth)
    }

    pub fn block0_payload_builder(payload: &P) -> TxBuilderState<SetAuthData<P>>
    where
        P: Payload,
    {
        TxBuilder::new()
            .set_payload(payload)
            .set_expiry_date(BlockDate::first().next_epoch())
            .set_ios(&[], &[])
            .set_witnesses(&[])
    }

    // pretend that the construction doesn't enforce #inputs == #witness by
    // exposing another accessor for general purpose
    pub fn nb_witnesses(&self) -> u8 {
        self.tstruct.nb_inputs
    }
    pub fn nb_outputs(&self) -> u8 {
        self.tstruct.nb_outputs
    }

    pub fn total_input(&self) -> Result<Value, ValueError> {
        Value::sum(self.as_slice().inputs().iter().map(|input| input.value()))
    }

    pub fn total_output(&self) -> Result<Value, ValueError> {
        Value::sum(self.as_slice().outputs().iter().map(|output| output.value))
    }

    pub fn balance(&self, fee: Value) -> Result<Balance, ValueError> {
        use std::cmp::Ordering::*;

        let inputs = self.total_input()?;
        let outputs = self.total_output()?;
        let z = (outputs + fee)?;

        match inputs.cmp(&z) {
            Greater => Ok(Balance::Positive((inputs - z)?)),
            Less => Ok(Balance::Negative((z - inputs)?)),
            Equal => Ok(Balance::Zero),
        }
    }

    pub fn verify_strictly_balanced(&self, fee: Value) -> Result<(), BalanceError> {
        let inputs = self
            .total_input()
            .map_err(BalanceError::InputsTotalFailed)?;
        let outputs = self
            .total_output()
            .and_then(|out| out + fee)
            .map_err(BalanceError::OutputsTotalFailed)?;
        if inputs != outputs {
            return Err(BalanceError::NotBalanced { inputs, outputs });
        };
        Ok(())
    }

    pub fn verify_possibly_balanced(&self) -> Result<(), BalanceError> {
        let inputs = self
            .total_input()
            .map_err(BalanceError::InputsTotalFailed)?;
        let outputs = self
            .total_output()
            .map_err(BalanceError::OutputsTotalFailed)?;
        if inputs < outputs {
            return Err(BalanceError::NotBalanced { inputs, outputs });
        };
        Ok(())
    }
}

impl<'a, P> TransactionSlice<'a, P> {
    pub fn to_owned(&self) -> Transaction<P> {
        let mut data = Vec::with_capacity(self.data.len());
        data.extend_from_slice(self.data);
        Transaction {
            data: data.into(),
            tstruct: self.tstruct.clone(),
            phantom: self.phantom,
        }
    }

    pub fn transaction_auth_data(&self) -> TransactionAuthData<'a> {
        TransactionAuthData(&self.data[0..self.tstruct.witnesses])
    }

    pub fn transaction_sign_data_hash(&self) -> TransactionSignDataHash {
        Digest::digest(self.transaction_auth_data().0).into()
    }

    pub fn transaction_binding_auth_data(&self) -> TransactionBindingAuthData<'a> {
        TransactionBindingAuthData(&self.data[0..self.tstruct.payload_auth])
    }

    pub fn payload(&self) -> PayloadSlice<'a, P> {
        PayloadSlice(&self.data[0..self.tstruct.inputs], PhantomData)
    }

    pub fn nb_inputs(&self) -> u8 {
        self.tstruct.nb_inputs
    }

    pub fn valid_until(&self) -> BlockDate {
        self.tstruct.valid_until
    }

    pub fn nb_outputs(&self) -> u8 {
        self.tstruct.nb_outputs
    }

    // pretend that the construction doesn't enforce #inputs == #witness by
    // exposing another accessor for general purpose
    pub fn nb_witnesses(&self) -> u8 {
        self.tstruct.nb_inputs
    }

    pub fn inputs(&self) -> InputsSlice<'a> {
        InputsSlice(
            self.tstruct.nb_inputs,
            &self.data[self.tstruct.inputs..self.tstruct.outputs],
        )
    }

    pub fn outputs(&self) -> OutputsSlice<'a> {
        OutputsSlice(
            self.tstruct.nb_outputs,
            &self.data[self.tstruct.outputs..self.tstruct.witnesses],
        )
    }

    pub fn witnesses(&self) -> WitnessesSlice<'a> {
        WitnessesSlice(
            self.tstruct.nb_inputs,
            &self.data[self.tstruct.witnesses..self.tstruct.payload_auth],
        )
    }

    pub fn inputs_and_witnesses(&self) -> InputsWitnessesSlice<'a> {
        InputsWitnessesSlice(self.inputs(), self.witnesses())
    }

    pub fn payload_auth(&self) -> PayloadAuthSlice<'a, P>
    where
        P: Payload,
    {
        PayloadAuthSlice(&self.data[self.tstruct.payload_auth..], PhantomData)
    }

    pub fn total_input(&self) -> Result<Value, ValueError> {
        Value::sum(self.inputs().iter().map(|input| input.value()))
    }

    pub fn total_output(&self) -> Result<Value, ValueError> {
        Value::sum(self.outputs().iter().map(|output| output.value))
    }

    pub fn verify_strictly_balanced(&self, fee: Value) -> Result<(), BalanceError> {
        let inputs = self
            .total_input()
            .map_err(BalanceError::InputsTotalFailed)?;
        let outputs = self
            .total_output()
            .and_then(|out| out + fee)
            .map_err(BalanceError::OutputsTotalFailed)?;
        if inputs != outputs {
            return Err(BalanceError::NotBalanced { inputs, outputs });
        };
        Ok(())
    }
}

impl<P> AsRef<[u8]> for Transaction<P> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}
