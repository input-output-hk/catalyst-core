use chain_impl_mockchain::config::Block0Date;
use chain_impl_mockchain::key::Hash;
use chain_time::SlotDuration;
use chain_time::TimeFrame;
use chain_time::Timeline;
use chain_time::{Epoch, TimeEra};
use jormungandr_lib::interfaces::SettingsDto;
use std::str::FromStr;
use wallet::Settings;

pub trait SettingsExtensions {
    fn into_wallet_settings(self) -> Settings;
}

impl SettingsExtensions for SettingsDto {
    fn into_wallet_settings(self) -> Settings {
        let duration_since_epoch = self.block0_time.duration_since_epoch();
        let timeline = Timeline::new(std::time::SystemTime::now());
        let tf = TimeFrame::new(timeline, SlotDuration::from_secs(self.slot_duration as u32));
        let slot0 = tf.slot0();
        let era = TimeEra::new(slot0, Epoch(0), self.slots_per_epoch);

        Settings {
            fees: self.fees,
            discrimination: self.discrimination,
            block0_initial_hash: Hash::from_str(&self.block0_hash).unwrap(),
            block0_date: Block0Date(duration_since_epoch.as_secs()),
            slot_duration: self.slot_duration as u8,
            time_era: era,
            transaction_max_expiry_epochs: self.tx_max_expiry_epochs,
        }
    }
}
