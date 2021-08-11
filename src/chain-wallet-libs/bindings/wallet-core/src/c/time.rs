use crate::{Error, Result};
use std::time::{Duration, SystemTime};
use wallet::Settings;

#[repr(C)]
pub struct BlockDate {
    pub epoch: u32,
    pub slot: u32,
}

impl From<chain_impl_mockchain::block::BlockDate> for BlockDate {
    fn from(date: chain_impl_mockchain::block::BlockDate) -> Self {
        BlockDate {
            epoch: date.epoch,
            slot: date.slot_id,
        }
    }
}

impl From<BlockDate> for chain_impl_mockchain::block::BlockDate {
    fn from(date: BlockDate) -> Self {
        chain_impl_mockchain::block::BlockDate {
            epoch: date.epoch,
            slot_id: date.slot,
        }
    }
}

///
/// # Safety
///
/// settings should be a pointer to a valid settings object allocated by this library with, for
/// example, settings_build.
pub unsafe fn block_date_from_system_time(
    settings: *const Settings,
    date: u64,
    block_date_out: *mut BlockDate,
) -> Result {
    let settings = non_null!(settings);
    match wallet::time::block_date_from_system_time(
        settings,
        SystemTime::UNIX_EPOCH + Duration::from_secs(date),
    ) {
        Ok(block_date) => {
            (*block_date_out).epoch = block_date.epoch;
            (*block_date_out).slot = block_date.slot_id;

            Result::success()
        }
        Err(_) => Error::invalid_transaction_validity_date().into(),
    }
}

///
/// # Safety
///
/// settings should be a pointer to a valid settings object allocated by this library with, for
/// example, settings_build.
pub unsafe fn max_epiration_date(
    settings: *const Settings,
    current_time: u64,
    block_date_out: *mut BlockDate,
) -> Result {
    let settings = non_null!(settings);
    match wallet::time::max_expiration_date(
        settings,
        SystemTime::UNIX_EPOCH + Duration::from_secs(current_time),
    ) {
        Ok(block_date) => {
            (*block_date_out).epoch = block_date.epoch;
            (*block_date_out).slot = block_date.slot_id;

            Result::success()
        }
        Err(_) => Error::invalid_transaction_validity_date().into(),
    }
}
