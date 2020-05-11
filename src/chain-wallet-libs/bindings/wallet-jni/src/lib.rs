use jni::objects::{JClass, JString};
use jni::sys::{jbyte, jbyteArray, jint, jlong};
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

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
        0
    } else {
        wallet as jlong
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
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
) -> jint {
    let wallet_ptr: WalletPtr = wallet as WalletPtr;
    let mut value: u64 = 0;
    if !wallet_ptr.is_null() {
        let result = wallet_total_value(wallet_ptr, &mut value);

        if let Some(error) = result.error() {
            let _ = env.throw(error.to_string());
        }
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
        let result =
            wallet_retrieve_funds(wallet_ptr, bytes.as_ptr() as *const u8, len, settings_ptr);
        if let Some(error) = result.error() {
            let _ = env.throw(error.to_string());
        }
    }
    settings as jlong
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_convert(
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
    settings: jlong,
) -> jlong {
    let wallet_ptr = wallet as WalletPtr;
    let settings_ptr = settings as SettingsPtr;
    let mut conversion_out = null_mut();
    let result = wallet_convert(
        wallet_ptr,
        settings_ptr,
        (&mut conversion_out) as *mut ConversionPtr,
    );

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
    }

    conversion_out as jlong
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Conversion_transactionsSize(
    _env: JNIEnv,
    _: JClass,
    conversion: ConversionPtr,
) -> jint {
    let conversion = conversion as ConversionPtr;
    // TODO: i don't think this can actually overflow, but probably best to have someone confirm that too
    wallet_convert_transactions_size(conversion) as i32
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Conversion_transactionsGet(
    env: JNIEnv,
    _: JClass,
    conversion: ConversionPtr,
    index: jint,
) -> jbyteArray {
    let conversion = conversion as ConversionPtr;

    if index.is_negative() {
        let _ = env.throw_new(
            "java/lang/IndexOutOfBoundsException",
            "Conversion transaction index should be a positive number",
        );
        return null_mut();
    }

    let index = index as usize;
    let mut transaction_out: *const u8 = null();
    let mut transaction_size: usize = 0;

    let result = wallet_convert_transactions_get(
        conversion,
        index,
        (&mut transaction_out) as *mut *const u8,
        (&mut transaction_size) as *mut usize,
    );

    // TODO: maybe we can use a ByteBuffer or something here to avoid the copy
    // (especially considering that there is already a requirement to call conversion's delete)
    let array = env
        .new_byte_array(transaction_size as jint)
        .expect("Failed to create new byte array");
    match result.error() {
        None => {
            let slice =
                std::slice::from_raw_parts(transaction_out as *const jbyte, transaction_size);
            env.set_byte_array_region(array, 0, slice)
                .expect("Couldn't copy array to jvm");
        }
        Some(error) => {
            let _ = env.throw(error.to_string());
        }
    };

    array
}

// TODO: get convert ignored values

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Conversion_delete(
    _: JNIEnv,
    _: JClass,
    conversion: jlong,
) {
    let conversion = conversion as ConversionPtr;
    if !conversion.is_null() {
        wallet_delete_conversion(conversion)
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_setState(
    env: JNIEnv,
    _: JClass,
    wallet: WalletPtr,
    value: jlong,
    counter: jlong,
) {
    let wallet = wallet as WalletPtr;
    let r = wallet_set_state(wallet, value as u64, counter as u32);

    if let Some(error) = r.error() {
        let _ = env.throw(error.to_string());
    }
}
