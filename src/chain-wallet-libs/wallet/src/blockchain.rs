use chain_impl_mockchain::{
    block::Block,
    ledger::{Error, Ledger, LedgerParameters, LedgerStaticParameters},
};

pub struct Settings {
    pub static_parameters: LedgerStaticParameters,
    pub parameters: LedgerParameters,
}

impl Settings {
    pub fn new(block: &Block) -> Result<Self, Error> {
        let header_id = block.header.id();
        let ledger = Ledger::new(header_id, block.contents.iter())?;

        let static_parameters = ledger.get_static_parameters().clone();
        let parameters = ledger.get_ledger_parameters();

        Ok(Self {
            static_parameters,
            parameters,
        })
    }
}
