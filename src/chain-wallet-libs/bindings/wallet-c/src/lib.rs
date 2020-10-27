pub use chain_impl_mockchain::vote::PayloadType as PayloadTypeRust;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};
pub use wallet::Settings as SettingsRust;
use wallet_core::c::{
    symmetric_cipher_decrypt, vote, wallet_convert, wallet_convert_ignored,
    wallet_convert_transactions_get, wallet_convert_transactions_size, wallet_delete_conversion,
    wallet_delete_error, wallet_delete_proposal, wallet_delete_settings, wallet_delete_wallet,
    wallet_id, wallet_import_keys, wallet_recover, wallet_retrieve_funds, wallet_set_state,
    wallet_total_value, wallet_vote_cast,
};
use wallet_core::{
    Conversion as ConversionRust, Error as ErrorRust, Proposal as ProposalRust,
    Wallet as WalletRust,
};

#[repr(C)]
pub struct Wallet {}
#[repr(C)]
pub struct Settings {}
#[repr(C)]
pub struct Conversion {}
#[repr(C)]
pub struct Proposal {}
#[repr(C)]
pub struct Error {}

pub type WalletPtr = *mut Wallet;
pub type SettingsPtr = *mut Settings;
pub type ConversionPtr = *mut Conversion;
pub type ProposalPtr = *mut Proposal;
pub type ErrorPtr = *mut Error;

/// Payload type for voting
#[repr(u8)]
pub enum PayloadType {
    Public = 1,
}

impl From<PayloadType> for PayloadTypeRust {
    fn from(c_enum: PayloadType) -> Self {
        match c_enum {
            PayloadType::Public => PayloadTypeRust::Public,
        }
    }
}

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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
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
    let r = wallet_recover(
        &mnemonics,
        password,
        password_length,
        wallet_out as *mut *mut WalletRust,
    );

    r.into_c_api() as ErrorPtr
}

/// recover a wallet from an account and a list of utxo keys
///
/// You can also use this function to recover a wallet even after you have
/// transferred all the funds to the new format (see the _convert_ function)
///
/// The recovered wallet will be returned in `wallet_out`.
///
/// # parameters
///
/// * account_key: the Ed25519 extended key used wallet's account address private key
///     in the form of a 64 bytes array.
/// * utxo_keys: an array of Ed25519 extended keys in the form of 64 bytes, used as utxo
///     keys for the wallet
/// * utxo_keys_len: the number of keys in the utxo_keys array (not the number of bytes)
/// * wallet_out: the recovered wallet
///
/// # Safety
///
/// This function dereference raw pointers (password and wallet_out). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
/// # errors
///
/// The function may fail if:
///
/// * the `wallet_out` is null pointer
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_import_keys(
    account_key: *const u8,
    utxo_keys: *const u8,
    utxo_keys_len: usize,
    wallet_out: *mut WalletPtr,
) -> ErrorPtr {
    let r = wallet_import_keys(
        account_key,
        utxo_keys as *const [u8; 64],
        utxo_keys_len,
        wallet_out as *mut *mut WalletRust,
    );

    r.into_c_api() as ErrorPtr
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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
/// the `ErrorPtr` from memory and avoid memory leaks.
///
/// * this function may fail if the wallet pointer is null;
///
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_id(
    wallet: WalletPtr,
    id_out: *mut u8,
) -> ErrorPtr {
    wallet_id(wallet as *mut WalletRust, id_out).into_c_api() as ErrorPtr
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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
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
    let r = wallet_retrieve_funds(
        wallet as *mut WalletRust,
        block0,
        block0_length,
        settings_out as *mut *mut SettingsRust,
    );

    r.into_c_api() as ErrorPtr
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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
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
    let r = wallet_convert(
        wallet as *mut WalletRust,
        settings as *mut SettingsRust,
        conversion_out as *mut *mut ConversionRust,
    );

    r.into_c_api() as ErrorPtr
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
    wallet_convert_transactions_size(conversion as *mut ConversionRust)
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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
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
    let r = wallet_convert_transactions_get(
        conversion as *mut ConversionRust,
        index,
        transaction_out,
        transaction_size,
    );

    r.into_c_api() as ErrorPtr
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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
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
    let r = wallet_convert_ignored(conversion as *mut ConversionRust, value_out, ignored_out);

    r.into_c_api() as ErrorPtr
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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
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
    let r = wallet_total_value(wallet as *mut WalletRust, total_out);

    r.into_c_api() as ErrorPtr
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
/// Don't forget to call `iohk_jormungandr_wallet_delete_error` to free
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
    let r = wallet_set_state(wallet as *mut WalletRust, value, counter);

    r.into_c_api() as ErrorPtr
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
pub unsafe extern "C" fn iohk_jormungandr_vote_proposal_new_public(
    vote_plan_id: *const u8,
    index: u8,
    num_choices: u8,
    proposal_out: *mut ProposalPtr,
) -> ErrorPtr {
    let r = vote::proposal_new(
        vote_plan_id,
        index,
        num_choices,
        vote::ProposalPublic,
        proposal_out as *mut *mut ProposalRust,
    );

    r.into_c_api() as ErrorPtr
}

/// build the proposal object
///
/// * `vote_encryption_key`: the vote encryption key argument is expected
/// to be a 65 bytes array
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
pub unsafe extern "C" fn iohk_jormungandr_vote_proposal_new_private(
    vote_plan_id: *const u8,
    index: u8,
    num_choices: u8,
    vote_encryption_key: *const u8,
    proposal_out: *mut ProposalPtr,
) -> ErrorPtr {
    let r = vote::proposal_new(
        vote_plan_id,
        index,
        num_choices,
        vote::ProposalPrivate(vote_encryption_key),
        proposal_out as *mut *mut ProposalRust,
    );

    r.into_c_api() as ErrorPtr
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
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_vote_cast(
    wallet: WalletPtr,
    settings: SettingsPtr,
    proposal: ProposalPtr,
    choice: u8,
    transaction_out: *mut *const u8,
    len_out: *mut usize,
) -> ErrorPtr {
    let r = wallet_vote_cast(
        wallet as *mut WalletRust,
        settings as *mut SettingsRust,
        proposal as *mut ProposalRust,
        choice,
        transaction_out,
        len_out,
    );

    r.into_c_api() as ErrorPtr
}

/// decrypt payload of the wallet transfer protocol
///
/// Parameters
///
/// password: byte buffer with the encryption password
/// password_length: length of the password buffer
/// ciphertext: byte buffer with the encryption password
/// ciphertext_length: length of the password buffer
/// plaintext_out: used to return a pointer to a byte buffer with the decrypted text
/// plaintext_out_length: used to return the length of decrypted text
///
/// The returned buffer is in the heap, so make sure to call the delete_buffer function
///
/// # Safety
///
/// This function dereference raw pointers. Even though the function checks if
/// the pointers are null. Mind not to put random values in or you may see
/// unexpected behaviors.
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_symmetric_cipher_decrypt(
    password: *const u8,
    password_length: usize,
    ciphertext: *const u8,
    ciphertext_length: usize,
    plaintext_out: *mut *const u8,
    plaintext_out_length: *mut usize,
) -> ErrorPtr {
    let r = symmetric_cipher_decrypt(
        password,
        password_length,
        ciphertext,
        ciphertext_length,
        plaintext_out,
        plaintext_out_length,
    );

    r.into_c_api() as ErrorPtr
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
    if let Some(error) = (error as *mut ErrorRust).as_ref() {
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
    if let Some(error) = (error as *mut ErrorRust).as_ref() {
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

/// Delete a binary buffer that was returned by this library alongside with its
/// length.
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
#[no_mangle]
pub unsafe extern "C" fn iohk_jormungandr_wallet_delete_buffer(ptr: *mut u8, length: usize) {
    if !ptr.is_null() {
        let data = std::slice::from_raw_parts_mut(ptr, length);
        let data = Box::from_raw(data as *mut [u8]);
        std::mem::drop(data);
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
    wallet_delete_error(error as *mut ErrorRust)
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
    wallet_delete_settings(settings as *mut SettingsRust)
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
    wallet_delete_wallet(wallet as *mut WalletRust)
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
    wallet_delete_conversion(conversion as *mut ConversionRust)
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
    wallet_delete_proposal(proposal as *mut ProposalRust)
}
