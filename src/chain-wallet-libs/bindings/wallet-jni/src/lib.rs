use jni::objects::{JClass, JString};
use jni::sys::{jbyteArray, jint, jlong};
use jni::JNIEnv;
use std::ptr::{null, null_mut};
use wallet_core::c::*;

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_recover(
    env: JNIEnv,
    _: JClass,
    mnemonics: JString,
) -> jlong {
    let mnemonics_j = env
        .get_string(mnemonics)
        .expect("Couldn't get mnemonics String");

    let mut wallet: WalletPtr = null_mut();
    let wallet_ptr: *mut WalletPtr = &mut wallet;
    let result = wallet_recover(&mnemonics_j.to_string_lossy(), null(), 0, wallet_ptr);

    let _r = env.release_string_utf_chars(mnemonics, mnemonics_j.as_ptr());

    if result.is_ok() {
        wallet as jlong
    } else {
        0
    }
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_delete(
    _: JNIEnv,
    _: JClass,
    wallet: jlong,
) {
    let wallet_ptr: WalletPtr = wallet as WalletPtr;
    if !wallet_ptr.is_null() {
        wallet_delete_wallet(wallet_ptr);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Settings_delete(
    _: JNIEnv,
    _: JClass,
    settings: jlong,
) {
    let settings_ptr: SettingsPtr = settings as SettingsPtr;
    if !settings_ptr.is_null() {
        wallet_delete_settings(settings_ptr);
    }
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_totalValue(
    _: JNIEnv,
    _: JClass,
    wallet: jlong,
) -> jint {
    let wallet_ptr: WalletPtr = wallet as WalletPtr;
    let mut value: u64 = 0;
    if !wallet_ptr.is_null() {
        wallet_total_value(wallet_ptr, &mut value);
    }
    value as jint
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_initialFunds(
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
    block0: jbyteArray,
) -> jlong {
    let wallet_ptr: WalletPtr = wallet as WalletPtr;
    let mut settings: SettingsPtr = null_mut();
    let settings_ptr: *mut SettingsPtr = &mut settings;
    let len = env
        .get_array_length(block0)
        .expect("Couldn't get block0 array length") as usize;
    let mut bytes = vec![0i8; len as usize];
    let _r = env.get_byte_array_region(block0, 0, &mut bytes);

    if !wallet_ptr.is_null() {
        wallet_retrieve_funds(wallet_ptr, bytes.as_ptr() as *const u8, len, settings_ptr);
    }
    settings as jlong
}
