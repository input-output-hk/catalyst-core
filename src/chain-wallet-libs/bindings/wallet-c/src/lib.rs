use std::{ffi::CStr, os::raw::c_char};
use wallet_core::c::*;

/// retrieve a wallet from the given mnemonics, password and protocol magic
///
/// this function will work for all yoroi, daedalus and other wallets
/// as it will try every kind of wallet anyway
///
/// You can also use this function to recover a wallet even after you have
/// transferred all the funds to the new format (see the _convert_ function)
///
/// The recovered wallet will be returned in `wallet_out`.
///
/// # parameters
///
/// * mnemonics: a null terminated utf8 string (already normalized NFKD) in english;
/// * password: pointer to the password (in bytes, can be UTF8 string or a bytes of anything);
///   this value is optional and passing a null pointer will result in no password;
/// * password_length: the length of the password;
/// * wallet_out: a pointer to a pointer. The recovered wallet will be allocated on this pointer;
///
/// # errors
///
/// The function may fail if:
///
/// * the mnemonics are not valid (invalid length or checksum);
/// * the `wallet_out` is null pointer
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_recover(
    mnemonics: *const c_char,
    password: *const u8,
    password_length: usize,
    wallet_out: *mut WalletPtr,
) -> ResultPtr {
    let mnemonics = CStr::from_ptr(mnemonics);

    let mnemonics = mnemonics.to_string_lossy();
    let r = wallet_recover(&mnemonics, password, password_length, wallet_out);

    if r.is_ok() {
        std::ptr::null_mut()
    } else {
        Box::into_raw(Box::new(r))
    }
}

/// retrieve funds from daedalus or yoroi wallet in the given block0 (or
/// any other blocks).
///
/// Execute this function then you can check who much funds you have
/// retrieved from the given block.
///
/// this function may take sometimes so it is better to only call this
/// function if needed.
///
/// # Parameters
///
/// * wallet: the recovered wallet (see recover function);
/// * block0: the pointer to the bytes of the block0;
/// * block0_length: the length of the block0 byte string;
/// * settings_out: the settings that will be parsed from the given
///   block0;
///
/// # Errors
///
/// * this function may fail if the wallet pointer is null;
/// * the block is not valid (cannot be decoded)
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_retrieve_funds(
    wallet: WalletPtr,
    block0: *const u8,
    block0_length: usize,
    settings_out: *mut SettingsPtr,
) -> ResultPtr {
    let r = wallet_retrieve_funds(wallet, block0, block0_length, settings_out);

    if r.is_ok() {
        std::ptr::null_mut()
    } else {
        Box::into_raw(Box::new(r))
    }
}

/// once funds have been retrieved with `iohk_jormungandr_wallet_retrieve_funds`
/// it is possible to convert all existing funds to the new wallet.
///
/// The returned arrays are transactions to send to the network in order to do the
/// funds conversion.
///
/// Don't forget to call `iohk_jormungandr_wallet_delete_conversion` to
/// properly free the memory
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_convert(
    wallet: WalletPtr,
    settings: SettingsPtr,
    conversion_out: *mut ConversionPtr,
) -> ResultPtr {
    let r = wallet_convert(wallet, settings, conversion_out);

    if r.is_ok() {
        std::ptr::null_mut()
    } else {
        Box::into_raw(Box::new(r))
    }
}

/// get the number of transactions built to convert the retrieved wallet
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_convert_transactions_size(
    conversion: ConversionPtr,
) -> usize {
    wallet_convert_transactions_size(conversion)
}

/// retrieve the index-nth transactions in the conversions starting from 0
/// and finishing at `size-1` where size is retrieved from
/// `iohk_jormungandr_wallet_convert_transactions_size`.
///
/// the memory allocated returned is not owned and should not be kept
/// for longer than potential call to `iohk_jormungandr_wallet_delete_conversion`
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_convert_transactions_get(
    conversion: ConversionPtr,
    index: usize,
    transaction_out: *mut *const u8,
    transaction_size: *mut usize,
) -> ResultPtr {
    let r = wallet_convert_transactions_get(conversion, index, transaction_out, transaction_size);

    if r.is_ok() {
        std::ptr::null_mut()
    } else {
        Box::into_raw(Box::new(r))
    }
}

/// get the total value ignored in the conversion
///
/// value_out: will returns the total value lost into dust inputs
/// ignored_out: will returns the number of dust utxos
///
/// these returned values are informational only and this show that
/// there are UTxOs entries that are unusable because of the way they
/// are populated with dusts.
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_convert_ignored(
    conversion: ConversionPtr,
    value_out: *mut u64,
    ignored_out: *mut usize,
) -> ResultPtr {
    let r = wallet_convert_ignored(conversion, value_out, ignored_out);

    if r.is_ok() {
        std::ptr::null_mut()
    } else {
        Box::into_raw(Box::new(r))
    }
}

/// get the total value in the wallet
///
/// make sure to call `retrieve_funds` prior to calling this function
/// otherwise you will always have `0`
///
/// After calling this function the results is returned in the `total_out`.
///
/// # Errors
///
/// * this function may fail if the wallet pointer is null;
///
/// If the `total_out` pointer is null, this function does nothing
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_total_value(
    wallet: WalletPtr,
    total_out: *mut u64,
) -> ResultPtr {
    let r = wallet_total_value(wallet, total_out);

    if r.is_ok() {
        std::ptr::null_mut()
    } else {
        Box::into_raw(Box::new(r))
    }
}

/// update the wallet account state
///
/// this is the value retrieved from any jormungandr endpoint that allows to query
/// for the account state. It gives the value associated to the account as well as
/// the counter.
///
/// It is important to be sure to have an updated wallet state before doing any
/// transactions otherwise future transactions may fail to be accepted by any
/// nodes of the blockchain because of invalid signature state.
///
/// # Errors
///
/// * this function may fail if the wallet pointer is null;
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_set_state(
    wallet: WalletPtr,
    value: u64,
    counter: u32,
) -> ResultPtr {
    let r = wallet_set_state(wallet, value, counter);

    if r.is_ok() {
        std::ptr::null_mut()
    } else {
        Box::into_raw(Box::new(r))
    }
}

/// delete the pointer and free the allocated memory
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_result(result: ResultPtr) {
    wallet_delete_result(result)
}

/// delete the pointer and free the allocated memory
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_settings(settings: SettingsPtr) {
    wallet_delete_settings(settings)
}

/// delete the pointer, zero all the keys and free the allocated memory
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_wallet(wallet: WalletPtr) {
    wallet_delete_wallet(wallet)
}

/// delete the pointer
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_conversion(conversion: ConversionPtr) {
    wallet_delete_conversion(conversion)
}
