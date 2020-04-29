use chain_addr::Discrimination;
use chain_impl_mockchain::{
    block::Block,
    fee::FeeAlgorithm as _,
    ledger::{Error, Ledger, LedgerParameters, LedgerStaticParameters},
    transaction::Input,
    vote::CommitteeId,
};
use chain_time::TimeEra;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct Settings {
    pub static_parameters: LedgerStaticParameters,
    pub parameters: LedgerParameters,
    pub time_era: TimeEra,
}

impl Settings {
    pub fn new(block: &Block) -> Result<Self, Error> {
        let header_id = block.header.id();
        let ledger = Ledger::new(header_id, block.contents.iter())?;

        let static_parameters = ledger.get_static_parameters().clone();
        let parameters = ledger.get_ledger_parameters();
        let time_era = ledger.era().clone();

        Ok(Self {
            static_parameters,
            parameters,
            time_era,
        })
    }

    /// convenient function to check if a given input
    /// is covering at least its own input fees for a given transaction
    pub fn is_input_worth(&self, input: &Input) -> bool {
        let value = input.value();
        let minimal_value = self.parameters.fees.fees_for_inputs_outputs(1, 0);

        value > minimal_value
    }

    // extract the block0 date-time
    // seconds in unix time (seconds elapsed since 1-Jan-1970)
    pub fn start_date_time(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.static_parameters.block0_start_time.0)
    }

    pub fn discrimination(&self) -> Discrimination {
        self.static_parameters.discrimination
    }

    pub fn time_era(&self) -> &TimeEra {
        &self.time_era
    }

    pub fn committee(&self) -> &[CommitteeId] {
        self.parameters.committees.as_slice()
    }
}
