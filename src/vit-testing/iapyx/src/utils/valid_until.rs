use chain_impl_mockchain::block::BlockDate;
use std::time::{Duration, SystemTime};
use wallet::time::block_date_from_system_time;
use wallet::Settings;

pub enum ValidUntil {
    ByBlockDate(BlockDate),
    BySlotShift(u8),
}

impl ValidUntil {
    pub fn into_expiry_date(
        self,
        settings: Option<Settings>,
    ) -> Result<BlockDate, wallet::time::Error> {
        match self {
            Self::ByBlockDate(block_date) => Ok(block_date),
            Self::BySlotShift(slot_shift) => {
                let settings =
                    settings.expect("settings are required when calculatin valid until block date");
                let shift_in_seconds = settings.slot_duration * slot_shift;
                let date = SystemTime::UNIX_EPOCH + Duration::from_secs(shift_in_seconds.into());
                block_date_from_system_time(&settings, date)
            }
        }
    }
}

impl Default for ValidUntil {
    fn default() -> Self {
        Self::BySlotShift(10)
    }
}
