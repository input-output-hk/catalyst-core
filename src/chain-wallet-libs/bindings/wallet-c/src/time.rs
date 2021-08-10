use wallet_core::c::time::{compute_end_date, BlockDate};

use crate::{ErrorPtr, Settings};

/// This function dereference raw pointers. Even though the function checks if
/// the pointers are null. Mind not to put random values in or you may see
/// unexpected behaviors.
///
/// # Arguments
///
/// *settings*: the blockchain settings previously allocated with this library.  
/// *final_date*: desired date of expiration for a fragment. If 0 is passed then the expiration
/// date will be set to the maximum possible date.
/// *block_date_out*: pointer to an allocated BlockDate structure, the memory should be writable.
///
/// # Safety
///
/// pointers should be allocated by this library and be valid.
/// null pointers are checked and will result in an error.
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_compute_ttl_from_date(
    settings: *const Settings,
    final_date: u64,
    block_date_out: *mut BlockDate,
) -> ErrorPtr {
    let final_date = std::num::NonZeroU64::new(final_date);
    let r = compute_end_date(
        settings.cast::<wallet::Settings>(),
        final_date,
        block_date_out,
    );

    r.into_c_api() as ErrorPtr
}
