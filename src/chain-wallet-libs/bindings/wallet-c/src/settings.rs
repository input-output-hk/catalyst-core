use super::{ErrorPtr, SettingsPtr};
use wallet_core::c::settings::{
    settings_block0_hash, settings_discrimination, settings_fees, settings_new, Discrimination,
    LinearFee,
};

#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_new(
    linear_fee: LinearFee,
    discrimination: Discrimination,
    block_0_hash: *const u8,
    settings_out: *mut SettingsPtr,
) -> ErrorPtr {
    settings_new(
        linear_fee,
        discrimination,
        block_0_hash,
        settings_out as *mut *mut wallet_core::Settings,
    )
    .into_c_api() as ErrorPtr
}

#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_fees(
    settings: SettingsPtr,
    linear_fee_out: *mut LinearFee,
) -> ErrorPtr {
    settings_fees(settings as *const wallet_core::Settings, linear_fee_out).into_c_api() as ErrorPtr
}

#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_discrimination(
    settings: SettingsPtr,
    discrimination_out: *mut Discrimination,
) -> ErrorPtr {
    settings_discrimination(settings as *const wallet_core::Settings, discrimination_out)
        .into_c_api() as ErrorPtr
}

#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_settings_block0_hash(
    settings: SettingsPtr,
    block0_hash: *mut u8,
) -> ErrorPtr {
    settings_block0_hash(settings as *const wallet_core::Settings, block0_hash).into_c_api()
        as ErrorPtr
}
