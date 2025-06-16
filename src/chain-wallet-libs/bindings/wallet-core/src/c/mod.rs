//! This module expose handy C compatible functions to reuse in the different
//! C style bindings that we have (wallet-c, wallet-jni...)
#[macro_use]
mod macros;
pub mod fragment;
pub mod settings;
pub mod time;
pub mod vote;

use crate::{Error, Proposal, Result, Wallet};
use chain_impl_mockchain::{
    account::SpendingCounterIncreasing, fragment::Fragment, value::Value, vote::Choice,
};
use std::convert::TryInto;

use thiserror::Error;
pub use wallet::Settings;

use self::time::BlockDate;

pub type WalletPtr = *mut Wallet;
pub type SettingsPtr = *mut Settings;
pub type ProposalPtr = *mut Proposal;
pub type ErrorPtr = *mut Error;
pub type FragmentPtr = *mut Fragment;

#[derive(Debug, Error)]
#[error("null pointer")]
struct NulPtr;


pub const FRAGMENT_ID_LENGTH: usize = 32;
pub const NONCES_SIZE: usize = 8 * 4;

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
/// * utxo_keys: an array of Ed25519 keys in the form of 64 bytes, used as utxo
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
pub unsafe fn wallet_import_keys(account_key: *const u8, wallet_out: *mut WalletPtr) -> Result {
    let wallet_out = non_null_mut!(wallet_out);
    let account_key: &u8 = non_null!(account_key);

    let account_key: &[u8] = std::slice::from_raw_parts(account_key as *const u8, 64);

    let result = Wallet::recover_free_keys(account_key);

    match result {
        Ok(wallet) => {
            *wallet_out = Box::into_raw(Box::new(wallet));
            Result::success()
        }
        Err(err) => err.into(),
    }
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
/// This function dereference raw pointers (wallet and id_out). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
/// the `id_out` needs to be ready allocated 32bytes memory. If not this will result
/// in an undefined behavior, in the best scenario it will be a buffer overflow.
///
/// # Errors
///
/// * this function may fail if the wallet pointer is null;
///
pub unsafe fn wallet_id(wallet: WalletPtr, id_out: *mut u8) -> Result {
    let wallet: &Wallet = if let Some(wallet) = wallet.as_ref() {
        wallet
    } else {
        return Error::invalid_input("wallet").with(NulPtr).into();
    };
    if id_out.is_null() {
        return Error::invalid_input("id_out").with(NulPtr).into();
    }

    let id = wallet.id();

    let id_out = std::slice::from_raw_parts_mut(id_out, wallet::AccountId::SIZE);

    id_out.copy_from_slice(id.as_ref());

    Result::success()
}

/// Confirm the previously generated transaction identified by fragment_id
///
/// # Safety
///
/// This function dereference raw pointers (wallet, fragment_id). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors. It's also asummed that fragment_id is
/// a pointer to FRAGMENT_ID_LENGTH bytes of contiguous data.
///
pub unsafe fn wallet_confirm_transaction(wallet: WalletPtr, fragment_id: *const u8) -> Result {
    let wallet = non_null_mut!(wallet);
    let fragment_id: &u8 = non_null!(fragment_id);

    let fragment_id_bytes: [u8; FRAGMENT_ID_LENGTH] =
        std::slice::from_raw_parts(fragment_id as *const u8, FRAGMENT_ID_LENGTH)
            .try_into()
            .unwrap();

    wallet.confirm_transaction(fragment_id_bytes.into());

    Result::success()
}

/// get the current spending counter for the (only) account in this wallet
///
/// the memory can be deallocated with `spending_counters_delete`.
///
/// # Errors
///
/// * this function may fail if the wallet pointer is null;
///
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
pub unsafe fn wallet_spending_counters(
    wallet: WalletPtr,
    spending_counters_ptr_out: *mut SpendingCounters,
) -> Result {
    let wallet = non_null!(wallet);
    let spending_counters_out = non_null_mut!(spending_counters_ptr_out);

    let counters: Box<[u32]> = wallet.spending_counter().as_slice().into();

    let raw_ptr = Box::into_raw(counters);

    spending_counters_out.data = raw_ptr as *mut u32;

    Result::success()
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
pub unsafe fn wallet_total_value(wallet: WalletPtr, total_out: *mut u64) -> Result {
    let wallet = if let Some(wallet) = wallet.as_ref() {
        wallet
    } else {
        return Error::invalid_input("wallet").with(NulPtr).into();
    };

    if let Some(total_out) = total_out.as_mut() {
        let total = wallet.total_value();

        *total_out = *total.as_ref();
    }

    Result::success()
}

#[repr(C)]
pub struct SpendingCounters {
    pub data: *mut u32,
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
/// The wallet argument must be a pointer previously returned by this library. The data field in
/// nonces must be not null and point to an array of `len` size.
pub unsafe fn wallet_set_state(wallet: WalletPtr, value: u64, nonces: SpendingCounters) -> Result {
    let wallet = if let Some(wallet) = wallet.as_mut() {
        wallet
    } else {
        return Error::invalid_input("wallet").with(NulPtr).into();
    };

    let value = Value(value);

    let nonces = match std::slice::from_raw_parts_mut(nonces.data, SpendingCounterIncreasing::LANES)
        .try_into()
        .map_err(|_e| Error::invalid_spending_counters())
    {
        Ok(nonces) => nonces,
        Err(e) => return e.into(),
    };

    match wallet.set_state(value, nonces) {
        Ok(_) => Result::success(),
        Err(e) => e.into(),
    }
}

#[repr(C)]
pub struct TransactionOut {
    pub data: *const u8,
    pub len: usize,
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
pub unsafe fn wallet_vote_cast(
    wallet: WalletPtr,
    settings: SettingsPtr,
    proposal: ProposalPtr,
    choice: u8,
    valid_until: BlockDate,
    lane: u8,
    transaction_out: *mut TransactionOut,
) -> Result {
    let wallet = if let Some(wallet) = wallet.as_mut() {
        wallet
    } else {
        return Error::invalid_input("wallet").with(NulPtr).into();
    };

    let settings = if let Some(settings) = settings.as_ref() {
        settings.clone()
    } else {
        return Error::invalid_input("settings").with(NulPtr).into();
    };

    let proposal = if let Some(proposal) = proposal.as_ref() {
        proposal
    } else {
        return Error::invalid_input("proposal").with(NulPtr).into();
    };

    if transaction_out.is_null() {
        return Error::invalid_input("transaction_out").with(NulPtr).into();
    }

    let choice = Choice::new(choice);

    let transaction = match wallet.vote(settings, proposal, choice, &valid_until.into(), lane) {
        Ok(transaction) => Box::leak(transaction),
        Err(err) => return err.into(),
    };

    (*transaction_out).data = transaction.as_ptr();
    (*transaction_out).len = transaction.len();

    Result::success()
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
pub unsafe fn symmetric_cipher_decrypt(
    password: *const u8,
    password_length: usize,
    ciphertext: *const u8,
    ciphertext_length: usize,
    plaintext_out: *mut *const u8,
    plaintext_out_length: *mut usize,
) -> Result {
    let password = non_null!(password);
    let ciphertext = non_null!(ciphertext);

    let password = std::slice::from_raw_parts(password, password_length);
    let ciphertext = std::slice::from_raw_parts(ciphertext, ciphertext_length);

    match symmetric_cipher::decrypt(password, ciphertext) {
        Ok(plaintext) => {
            let len = plaintext.len();
            let ptr = Box::into_raw(plaintext);

            let out_len = non_null_mut!(plaintext_out_length);
            let out = non_null_mut!(plaintext_out);

            *out_len = len;
            *out = ptr as *const u8;

            Result::success()
        }
        Err(err) => Error::symmetric_cipher_error(err).into(),
    }
}

/// delete the pointer and free the allocated memory
///
/// # Safety
///
/// The point must have been previously returned by this library
pub unsafe fn wallet_delete_error(error: ErrorPtr) {
    if !error.is_null() {
        let boxed = Box::from_raw(error);

        std::mem::drop(boxed);
    }
}

/// delete the pointer and free the allocated memory
///
/// # Safety
///
/// The point must have been previously returned by this library
pub unsafe fn wallet_delete_settings(settings: SettingsPtr) {
    if !settings.is_null() {
        let boxed = Box::from_raw(settings);

        std::mem::drop(boxed);
    }
}

/// delete the pointer, zero all the keys and free the allocated memory
///
/// # Safety
///
/// The pointer must have been previously returned by this library
pub unsafe fn wallet_delete_wallet(wallet: WalletPtr) {
    if !wallet.is_null() {
        let boxed = Box::from_raw(wallet);

        std::mem::drop(boxed);
    }
}

/// delete the pointer
///
/// # Safety
///
/// The pointer must have been previously returned by this library
pub unsafe fn wallet_delete_proposal(proposal: ProposalPtr) {
    if !proposal.is_null() {
        let boxed = Box::from_raw(proposal);

        std::mem::drop(boxed);
    }
}

/// Release the memory holding the spending counters
///
/// # Safety
///
/// This function is safe as long as the structure returned by this library is not modified.
/// This function should only be called with a structure returned by this library.
pub unsafe fn spending_counters_delete(spending_counters: SpendingCounters) {
    let SpendingCounters { data } = spending_counters;
    if !data.is_null() {
        let data = std::slice::from_raw_parts_mut(data, SpendingCounterIncreasing::LANES);
        let data = Box::from_raw(data as *mut [u32]);
        std::mem::drop(data);
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
pub unsafe fn delete_buffer(ptr: *mut u8, length: usize) {
    if !ptr.is_null() {
        let data = std::slice::from_raw_parts_mut(ptr, length);
        let data = Box::from_raw(data as *mut [u8]);
        std::mem::drop(data);
    }
}
