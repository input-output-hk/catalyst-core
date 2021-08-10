use crate::Error;
use chain_impl_mockchain::block::BlockDate;
use std::time::{Duration, SystemTime};
use wallet::Settings;

pub fn compute_end_date(
    settings: &Settings,
    final_date: Option<std::num::NonZeroU64>,
) -> Result<BlockDate, Error> {
    wallet::time::compute_end_date(
        settings,
        final_date.map(|n| SystemTime::UNIX_EPOCH + Duration::from_secs(n.into())),
    )
    .map_err(|_| Error::invalid_transaction_validity_date())
}
