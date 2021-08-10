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
pub unsafe fn compute_end_date(
    settings: *const Settings,
    final_date: Option<std::num::NonZeroU64>,
    block_date_out: *mut BlockDate,
) -> Result {
    let settings = non_null!(settings);
    match wallet::time::compute_end_date(
        settings,
        final_date.map(|n| SystemTime::UNIX_EPOCH + Duration::from_secs(n.into())),
    ) {
        Ok(block_date) => {
            (*block_date_out).epoch = block_date.epoch;
            (*block_date_out).slot = block_date.slot_id;

            Result::success()
        }
        Err(_) => Error::invalid_transaction_validity_date().into(),
    }
}
