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
                let date = SystemTime::now() + Duration::from_secs(shift_in_seconds.into());
                block_date_from_system_time(&settings, date)
            }
        }
    }
}
/*
use chain_time::TimeFrame;
use chain_time::Timeline;
use chain_time::SlotDuration;

pub fn block_date_from_system_time(
    settings: &Settings,
    date: SystemTime,
) -> BlockDate {
    let start_time = SystemTime::UNIX_EPOCH + Duration::from_secs(settings.block0_date.0);
    let timeline = Timeline::new(start_time);
    let tf = TimeFrame::new(
        timeline.clone(),
        SlotDuration::from_secs(settings.slot_duration as u32),
    );
    println!("{:?}",timeline.differential(&date));
    let final_slot_offset = tf.slot_at(&date).unwrap();

    println!("left time era: {:?}",settings.time_era);
    println!("final_slot_offset: {:?}",final_slot_offset);

    let date = settings
        .time_era
        .from_slot_to_era(final_slot_offset)
        .unwrap();

    BlockDate {
        epoch: date.epoch.0,
        slot_id: date.slot.0,
    }
}
*/
impl Default for ValidUntil {
    fn default() -> Self {
        Self::BySlotShift(10)
    }
}
