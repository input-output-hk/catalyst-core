use crate::transaction::WitnessBuilder;
use chain_impl_mockchain::{transaction::Input, value::Value};

pub struct GeneratedInput {
    pub(crate) input: Input,
    pub(crate) witness_builder: WitnessBuilder,
}

/// input generator are essentially wallets.
///
/// For an account it will evaluate how the account can provide
/// funds to cover the required value
///
/// For a UTxO it will be based on how useful it is to implement
pub trait InputGenerator {
    fn input_to_cover(&mut self, value: Value) -> Option<GeneratedInput>;
}
