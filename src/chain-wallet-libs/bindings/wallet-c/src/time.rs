use wallet_core::c::time::{block_date_from_system_time, max_epiration_date, BlockDate};

use crate::{ErrorPtr, Settings};

/// This function dereference raw pointers. Even though the function checks if
/// the pointers are null. Mind not to put random values in or you may see
/// unexpected behaviors.
///
/// # Arguments
///
/// *settings*: the blockchain settings previously allocated with this library.  
/// *date*: desired date of expiration for a fragment. It must be expressed in seconds since the
/// unix epoch.
/// *block_date_out*: pointer to an allocated BlockDate structure, the memory should be writable.
///
/// # Safety
///
/// pointers should be allocated by this library and be valid.
/// null pointers are checked and will result in an error.
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_block_date_from_system_time(
    settings: *const Settings,
    date: u64,
    block_date_out: *mut BlockDate,
) -> ErrorPtr {
    let r = block_date_from_system_time(settings.cast::<wallet::Settings>(), date, block_date_out);

    r.into_c_api() as ErrorPtr
}

/// This function dereference raw pointers. Even though the function checks if
/// the pointers are null. Mind not to put random values in or you may see
/// unexpected behaviors.
///
/// # Arguments
///
/// *settings*: the blockchain settings previously allocated with this library.  
/// *current_time*: Current real time. It must be expressed in seconds since the unix epoch.
/// *block_date_out*: pointer to an allocated BlockDate structure, the memory should be writable.
///
/// # Safety
///
/// pointers should be allocated by this library and be valid.
/// null pointers are checked and will result in an error.
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_max_expiration_date(
    settings: *const Settings,
    current_time: u64,
    block_date_out: *mut BlockDate,
) -> ErrorPtr {
    let r = max_epiration_date(
        settings.cast::<wallet::Settings>(),
        current_time,
        block_date_out,
    );

    r.into_c_api() as ErrorPtr
}
