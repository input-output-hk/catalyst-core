use crate::Settings;
use chain_impl_mockchain::block::BlockDate;
use chain_time::{SlotDuration, TimeFrame, Timeline};
use std::time::{Duration, SystemTime};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("date is outside valid ttl range")]
    FinalDateOutOfRange,
    #[error("blockchain has not started")]
    BeforeBlock0Date,
}

pub fn compute_end_date(
    settings: &Settings,
    final_date: Option<SystemTime>,
) -> Result<BlockDate, Error> {
    let start_time = SystemTime::UNIX_EPOCH + Duration::from_secs(settings.block0_date.0);
    let timeline = Timeline::new(start_time);
    let tf = TimeFrame::new(
        timeline,
        SlotDuration::from_secs(settings.slot_duration as u32),
    );
    let current_time = SystemTime::now();
    let current_slot_offset = tf.slot_at(&current_time).ok_or(Error::BeforeBlock0Date)?;

    let current_date = settings
        .time_era
        .from_slot_to_era(current_slot_offset)
        .unwrap();

    let last_valid_epoch = current_date.epoch.0 + settings.transaction_max_expiry_epochs as u32;

    match final_date {
        Some(end_time) => {
            let final_slot_offset = tf.slot_at(&end_time).unwrap();

            let date = settings
                .time_era
                .from_slot_to_era(final_slot_offset)
                .unwrap();

            if date.epoch.0 > last_valid_epoch {
                Err(Error::FinalDateOutOfRange)
            } else {
                Ok(BlockDate {
                    epoch: date.epoch.0,
                    slot_id: date.slot.0,
                })
            }
        }
        None => Ok(BlockDate {
            epoch: last_valid_epoch,
            slot_id: current_date.slot.0,
        }),
    }
}

pub fn block_date_from_system_time(
    settings: &Settings,
    final_date: SystemTime,
) -> Result<BlockDate, Error> {
    let start_time = SystemTime::UNIX_EPOCH + Duration::from_secs(settings.block0_date.0);
    let timeline = Timeline::new(start_time);
    let tf = TimeFrame::new(
        timeline,
        SlotDuration::from_secs(settings.slot_duration as u32),
    );

    let final_slot_offset = tf.slot_at(&final_date).unwrap();

    let date = settings
        .time_era
        .from_slot_to_era(final_slot_offset)
        .unwrap();

    Ok(BlockDate {
        epoch: date.epoch.0,
        slot_id: date.slot.0,
    })
}

pub fn max_expiration_date(
    settings: &Settings,
    current_time: SystemTime,
) -> Result<BlockDate, Error> {
    let start_time = SystemTime::UNIX_EPOCH + Duration::from_secs(settings.block0_date.0);
    let timeline = Timeline::new(start_time);
    let tf = TimeFrame::new(
        timeline,
        SlotDuration::from_secs(settings.slot_duration as u32),
    );

    let current_slot_offset = tf.slot_at(&current_time).ok_or(Error::BeforeBlock0Date)?;

    let current_date = settings
        .time_era
        .from_slot_to_era(current_slot_offset)
        .unwrap();

    let last_valid_epoch = current_date.epoch.0 + settings.transaction_max_expiry_epochs as u32;

    Ok(BlockDate {
        epoch: last_valid_epoch,
        slot_id: settings
            .time_era
            .slots_per_epoch()
            .checked_sub(1)
            .expect("slots per epoch can't be zero"),
    })
}
