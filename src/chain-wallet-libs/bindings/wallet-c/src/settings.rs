use super::{ErrorPtr, SettingsPtr};
use wallet_core::c::settings::{
    settings_block0_hash, settings_discrimination, settings_fees, settings_new, Discrimination,
    LinearFee, SettingsInit,
};

/// # Safety
///
/// settings_out must point to valid writable memory
/// block_0_hash is assumed to point to 32 bytes of readable memory
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_new(
    settings_init: SettingsInit,
    settings_out: *mut SettingsPtr,
) -> ErrorPtr {
    settings_new(
        settings_init,
        settings_out as *mut *mut wallet_core::Settings,
    )
    .into_c_api() as ErrorPtr
}

/// # Safety
///
///   This function also assumes that settings is a valid pointer previously
///   obtained with this library, a null check is performed, but is important that
///   the data it points to is valid
///
///   linear_fee_out must point to valid writable memory, a null check is
///   performed
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_fees(
    settings: SettingsPtr,
    linear_fee_out: *mut LinearFee,
) -> ErrorPtr {
    settings_fees(settings as *const wallet_core::Settings, linear_fee_out).into_c_api() as ErrorPtr
}

/// # Safety
///
///   This function also assumes that settings is a valid pointer previously
///   obtained with this library, a null check is performed, but is important that
///   the data it points to is valid
///
///   discrimination_out must point to valid writable memory, a null check is
///   performed
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_discrimination(
    settings: SettingsPtr,
    discrimination_out: *mut Discrimination,
) -> ErrorPtr {
    settings_discrimination(settings as *const wallet_core::Settings, discrimination_out)
        .into_c_api() as ErrorPtr
}

/// # Safety
///
///   This function assumes block0_hash points to 32 bytes of valid memory
///   This function also assumes that settings is a valid pointer previously
///   obtained with this library, a null check is performed, but is important that
///   the data it points to is valid
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_block0_hash(
    settings: SettingsPtr,
    block0_hash: *mut u8,
) -> ErrorPtr {
    settings_block0_hash(settings as *const wallet_core::Settings, block0_hash).into_c_api()
        as ErrorPtr
}
