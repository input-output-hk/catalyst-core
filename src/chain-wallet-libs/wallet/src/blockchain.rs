use chain_addr::Discrimination;
use chain_impl_mockchain::{
    block::Block,
    config::{Block0Date, ConfigParam},
    fee::{FeeAlgorithm as _, LinearFee},
    fragment::Fragment,
    header::HeaderId,
    ledger::{Block0Error, Error, Ledger},
    transaction::Input,
};
use chain_time::TimeEra;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{Block0DateDef, DiscriminationDef, LinearFeeDef, TimeEraDef},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(with = "LinearFeeDef")]
    pub fees: LinearFee,
    #[serde(with = "DiscriminationDef")]
    pub discrimination: Discrimination,
    #[serde(with = "Hash")]
    pub block0_initial_hash: HeaderId,
    #[serde(with = "Block0DateDef")]
    pub block0_date: Block0Date,
    pub slot_duration: u8,
    #[serde(with = "TimeEraDef")]
    pub time_era: TimeEra,
    pub transaction_max_expiry_epochs: u8,
}

impl Settings {
    pub fn new(block: &Block) -> Result<Self, Error> {
        let header_id = block.header().id();
        let ledger = Ledger::new(header_id, block.contents().iter())?;

        let static_parameters = ledger.get_static_parameters().clone();
        let parameters = ledger.settings();

        // TODO: I think there is a bug in Ledger::new(), as it doesn't set the slot_duration in
        // the Settings.
        // This doesn't seem to matter for jormungandr anyway, because it gets the initial value
        // directly from the block0, and then just doesn't use the field in the settings anymore.
        // For now, just get the setting directly from the block0 here too, but should probably be
        // fixed in Ledger::new (or at least checked).
        let mut slot_duration = None;

        for fragment in block.contents().iter() {
            if let Fragment::Initial(initials) = fragment {
                for initial in initials.iter() {
                    if let ConfigParam::SlotDuration(sd) = initial {
                        slot_duration.replace(sd);
                    }
                }
            }
        }

        Ok(Self {
            fees: parameters.linear_fees.clone(),
            discrimination: static_parameters.discrimination,
            block0_initial_hash: static_parameters.block0_initial_hash,
            block0_date: static_parameters.block0_start_time,
            slot_duration: *slot_duration
                .ok_or(Error::Block0(Block0Error::InitialMessageNoSlotDuration))?,
            time_era: ledger.era().clone(),
            transaction_max_expiry_epochs: ledger.settings().transaction_max_expiry_epochs,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_serde() {
        use chain_time::Epoch;

        let settings = Settings {
            block0_date: Block0Date(0),
            fees: LinearFee::new(0, 0, 0),
            discrimination: Discrimination::Production,
            block0_initial_hash: HeaderId::zero_hash(),
            slot_duration: 0,
            transaction_max_expiry_epochs: 0,
            time_era: TimeEra::new(0_u64.into(), Epoch(0), 0),
        };
        
        let string = serde_json::to_string(&settings).unwrap();
        assert_eq!(string, "{\"fees\":{\"constant\":0,\"coefficient\":0,\"certificate\":0},\"discrimination\":\"production\",\"block0_initial_hash\":{\"hash\":\"0000000000000000000000000000000000000000000000000000000000000000\"},\"block0_date\":0,\"slot_duration\":0,\"time_era\":{\"epoch_start\":0,\"slot_start\":0,\"slots_per_epoch\":0},\"transaction_max_expiry_epochs\":0}");

        let decoded_settings: Settings = serde_json::from_str(&string).unwrap();
        assert_eq!(decoded_settings, settings);
    }
}
