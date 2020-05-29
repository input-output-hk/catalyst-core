use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};
pub use wallet::Settings;
use wallet_core::c::{
    wallet_convert, wallet_convert_ignored, wallet_convert_transactions_get,
    wallet_convert_transactions_size, wallet_delete_conversion, wallet_delete_error,
    wallet_delete_proposal, wallet_delete_settings, wallet_delete_wallet, wallet_id,
    wallet_recover, wallet_retrieve_funds, wallet_set_state, wallet_total_value, wallet_vote_cast,
    wallet_vote_proposal,
};
pub use wallet_core::{
    // c::{ConversionPtr, ErrorPtr, SettingsPtr, WalletPtr},
    Conversion,
    Error,
    ErrorCode,
    ErrorKind,
    Proposal,
    Wallet,
};

pub type WalletPtr = *mut Wallet;
pub type SettingsPtr = *mut Settings;
pub type ConversionPtr = *mut Conversion;
pub type ProposalPtr = *mut Proposal;
pub type ErrorPtr = *mut Error;

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
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
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
) -> ErrorPtr {
    let mnemonics = CStr::from_ptr(mnemonics);

    let mnemonics = mnemonics.to_string_lossy();
    let r = wallet_recover(&mnemonics, password, password_length, wallet_out);

    r.into_c_api()
}

/// get the wallet id
///
/// This ID is the identifier to use against the blockchain/explorer to retrieve
/// the state of the wallet (counter, total value etc...)
///
/// # Parameters
///
/// * wallet: the recovered wallet (see recover function);
/// * id_out: a ready allocated pointer to an array of 32bytes. If this array is not
///   32bytes this may result in a buffer overflow.
///
/// # Safety
///
/// This function dereference raw pointers (wallet, block0 and settings_out). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
/// the `id_out` needs to be ready allocated 32bytes memory. If not this will result
/// in an undefined behavior, in the best scenario it will be a buffer overflow.
///
/// # Errors
///
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
///
/// * this function may fail if the wallet pointer is null;
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_id(
    wallet: WalletPtr,
    id_out: *mut u8,
) -> ErrorPtr {
    wallet_id(wallet, id_out).into_c_api()
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
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
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
) -> ErrorPtr {
    let r = wallet_retrieve_funds(wallet, block0, block0_length, settings_out);

    r.into_c_api()
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
/// # Errors
///
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
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
) -> ErrorPtr {
    let r = wallet_convert(wallet, settings, conversion_out);

    r.into_c_api()
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
/// # Errors
///
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
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
) -> ErrorPtr {
    let r = wallet_convert_transactions_get(conversion, index, transaction_out, transaction_size);

    r.into_c_api()
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
/// # Errors
///
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
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
) -> ErrorPtr {
    let r = wallet_convert_ignored(conversion, value_out, ignored_out);

    r.into_c_api()
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
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
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
) -> ErrorPtr {
    let r = wallet_total_value(wallet, total_out);

    r.into_c_api()
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
/// On error the function returns a `ErrorPtr`. On success `NULL` is returned.
/// The `ErrorPtr` can then be observed to gathered details of the error.
/// Don't forget to call `iohk_jormungandr_wallet_delete_result` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
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
) -> ErrorPtr {
    let r = wallet_set_state(wallet, value, counter);

    r.into_c_api()
}

/// build the proposal object
///
/// # Errors
///
/// This function may fail if:
///
/// * `proposal_out` is null.
/// * `num_choices` is out of the allowed range.
///
/// # Safety
///
/// This function dereference raw pointers. Even though the function checks if
/// the pointers are null. Mind not to put random values in or you may see
/// unexpected behaviors.
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_vote_proposal(
    vote_plan_id: *const u8,
    payload_type: u8,
    index: u8,
    num_choices: u8,
    proposal_out: *mut ProposalPtr,
) -> ErrorPtr {
    let r = wallet_vote_proposal(vote_plan_id, payload_type, index, num_choices, proposal_out);

    r.into_c_api()
}

/// build the vote cast transaction
///
/// # Errors
///
/// This function may fail upon receiving a null pointer or a `choice` value
/// that does not fall within the range specified in `proposal`.
///
/// # Safety
///
/// This function dereference raw pointers. Even though the function checks if
/// the pointers are null. Mind not to put random values in or you may see
/// unexpected behaviors.
///
/// Don't forget to remove `transaction_out` with
/// `iohk_jormungandr_waller_delete_buffer`.
pub unsafe extern "C" fn iohk_jormungandr_wallet_vote_cast(
    wallet: WalletPtr,
    settings: SettingsPtr,
    proposal: ProposalPtr,
    choice: u8,
    transaction_out: *mut *const u8,
    len_out: *mut usize,
) -> ErrorPtr {
    let r = wallet_vote_cast(wallet, settings, proposal, choice, transaction_out, len_out);

    r.into_c_api()
}

/// Get a string describing the error, this will return an allocated
/// null terminated string describing the error.
///
/// If the given error is a `NULL` pointer, the string is and always
/// is `"success"`. This string still need to be deleted with the
/// `iohk_jormungandr_wallet_delete_string` function.
///
/// This function returns an allocated null terminated pointer. Don't
/// forget to free the memory with: `iohk_jormungandr_wallet_delete_string`.
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_error_to_string(error: ErrorPtr) -> *mut c_char {
    if let Some(error) = error.as_ref() {
        CString::new(error.to_string()).unwrap().into_raw()
    } else {
        CString::new(b"success".to_vec()).unwrap().into_raw()
    }
}

/// Get a string describing the error, this will return an allocated
/// null terminated string providing extra details regarding the source
/// of the error.
///
/// If the given error is a `NULL` pointer, the string is and always
/// is `"success"`. If no details are available the function will return
/// `"no more details"`. This string still need to be deleted with the
/// `iohk_jormungandr_wallet_delete_string` function.
///
/// This function returns an allocated null terminated pointer. Don't
/// forget to free the memory with: `iohk_jormungandr_wallet_delete_string`.
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_error_details(error: ErrorPtr) -> *mut c_char {
    if let Some(error) = error.as_ref() {
        if let Some(details) = error.details() {
            CString::new(details.to_string()).unwrap().into_raw()
        } else {
            CString::new(b"no more details".to_vec())
                .unwrap()
                .into_raw()
        }
    } else {
        CString::new(b"success".to_vec()).unwrap().into_raw()
    }
}

/// Delete a null terminated string that was allocated by this library
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_delete_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let cstring = CString::from_raw(ptr);
        std::mem::drop(cstring)
    }
}

/// Delete a binery buffer that was returned by this library alongside with its
/// length.
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_waller_delete_buffer(ptr: *mut c_char, length: usize) {
    if !ptr.is_null() {
        let vec = Vec::from_raw_parts(ptr, length, length);
        std::mem::drop(vec);
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
pub extern "C" fn iohk_jormungandr_wallet_delete_error(error: ErrorPtr) {
    wallet_delete_error(error)
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

/// delete the pointer
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_proposal(proposal: ProposalPtr) {
    wallet_delete_proposal(proposal)
}
