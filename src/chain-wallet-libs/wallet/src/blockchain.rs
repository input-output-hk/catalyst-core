use chain_addr::Discrimination;
use chain_impl_mockchain::{
    block::Block,
    fee::{FeeAlgorithm as _, LinearFee},
    header::HeaderId,
    ledger::{Error, Ledger},
    transaction::Input,
};

#[derive(Clone)]
pub struct Settings {
    pub fees: LinearFee,
    pub discrimination: Discrimination,
    pub block0_initial_hash: HeaderId,
}

impl Settings {
    pub fn new(block: &Block) -> Result<Self, Error> {
        let header_id = block.header.id();
        let ledger = Ledger::new(header_id, block.contents.iter())?;

        let static_parameters = ledger.get_static_parameters().clone();
        let parameters = ledger.get_ledger_parameters();

        Ok(Self {
            fees: parameters.fees,
            discrimination: static_parameters.discrimination,
            block0_initial_hash: static_parameters.block0_initial_hash,
        })
    }

    /// convenient function to check if a given input
    /// is covering at least its own input fees for a given transaction
    pub fn is_input_worth(&self, input: &Input) -> bool {
        let value = input.value();
        let minimal_value = self.fees.fees_for_inputs_outputs(1, 0);

        value > minimal_value
    }

    pub fn discrimination(&self) -> Discrimination {
        self.discrimination
    }
}
