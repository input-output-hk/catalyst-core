//! This module expose handy C compatible functions to reuse in the different
//! C style bindings that we have (wallet-c, wallet-jni...)
#[macro_use]
mod macros;
pub mod vote;

use crate::{Conversion, Error, Proposal, Result, Wallet};
use chain_impl_mockchain::{transaction::Input, value::Value, vote::Choice};
use std::convert::TryInto;

use thiserror::Error;
pub use wallet::Settings;

pub type WalletPtr = *mut Wallet;
pub type SettingsPtr = *mut Settings;
pub type ConversionPtr = *mut Conversion;
pub type ProposalPtr = *mut Proposal;
pub type ErrorPtr = *mut Error;
pub type PendingTransactionsPtr = *mut PendingTransactions;

#[derive(Debug, Error)]
#[error("null pointer")]
struct NulPtr;

#[derive(Debug, Error)]
#[error("access out of bound")]
struct OutOfBound;

/// opaque handle over a list of pending transaction ids
pub struct PendingTransactions {
    fragment_ids: Box<[chain_impl_mockchain::fragment::FragmentId]>,
}

pub const FRAGMENT_ID_LENGTH: usize = 32;

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
/// * the mnemonics are not valid (invalid length or checksum);
/// * the `wallet_out` is null pointer
///
pub unsafe fn wallet_recover(
    mnemonics: &str,
    password: *const u8,
    password_length: usize,
    wallet_out: *mut WalletPtr,
) -> Result {
    let wallet_out: &mut WalletPtr = if let Some(wallet_out) = wallet_out.as_mut() {
        wallet_out
    } else {
        return Error::invalid_input("wallet_out").with(NulPtr).into();
    };

    let result = if !password.is_null() && password_length > 0 {
        todo!()
    } else {
        Wallet::recover(mnemonics, &[])
    };

    match result {
        Ok(wallet) => {
            *wallet_out = Box::into_raw(Box::new(wallet));
            Result::success()
        }
        Err(err) => err.into(),
    }
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
pub unsafe fn wallet_import_keys(
    account_key: *const u8,
    utxo_keys: *const [u8; 64],
    utxo_keys_len: usize,
    wallet_out: *mut WalletPtr,
) -> Result {
    let wallet_out = non_null_mut!(wallet_out);
    let utxo_keys: &[u8; 64] = non_null!(utxo_keys);

    let account_key: &u8 = non_null!(account_key);
    let account_key: &[u8] = std::slice::from_raw_parts(account_key as *const u8, 64);

    let utxo_keys: &[[u8; 64]] =
        std::slice::from_raw_parts(utxo_keys.as_ptr() as *const [u8; 64], utxo_keys_len);

    let result = Wallet::recover_free_keys(account_key, &utxo_keys);

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

/// retrieve funds from daedalus or yoroi wallet in the given block0 (or
/// any other blocks).
///
/// Execute this function then you can check who much funds you have
/// retrieved from the given block.
///
/// this function may take sometimes so it is better to only call this
/// function if needed.
///
/// # Safety
///
/// This function dereference raw pointers (wallet, block0 and settings_out). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
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
pub unsafe fn wallet_retrieve_funds(
    wallet: WalletPtr,
    block0: *const u8,
    block0_length: usize,
    settings_out: *mut SettingsPtr,
) -> Result {
    let wallet: &mut Wallet = if let Some(wallet) = wallet.as_mut() {
        wallet
    } else {
        return Error::invalid_input("wallet").with(NulPtr).into();
    };
    if block0.is_null() {
        return Error::invalid_input("block0").with(NulPtr).into();
    }
    let settings_out: &mut SettingsPtr = if let Some(settings_out) = settings_out.as_mut() {
        settings_out
    } else {
        return Error::invalid_input("settings_out").with(NulPtr).into();
    };

    let block0_bytes = std::slice::from_raw_parts(block0, block0_length);

    match wallet.retrieve_funds(block0_bytes) {
        Ok(settings) => {
            *settings_out = Box::into_raw(Box::new(settings));
            Result::success()
        }
        Err(err) => err.into(),
    }
}

///
/// # Safety
///
/// This function dereference raw pointers (wallet, fragment_id). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors.
///
pub unsafe fn pending_transactions_len(
    transactions: PendingTransactionsPtr,
    len_out: *mut usize,
) -> Result {
    let pending_transactions = non_null!(transactions);

    *len_out = pending_transactions.fragment_ids.len();

    Result::success()
}

///
/// # Safety
///
/// This function dereference raw pointers (wallet, fragment_id). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors.
///
pub unsafe fn pending_transactions_get(
    transactions: PendingTransactionsPtr,
    index: usize,
    id_out: *mut *const u8,
) -> Result {
    let pending_transactions = non_null!(transactions);

    let fragment_id: &[u8] = pending_transactions.fragment_ids[index].as_ref();

    *id_out = fragment_id.as_ptr();

    Result::success()
}

/// delete the pointer and free the allocated memory
///
/// # Safety
///
/// This function dereference raw pointers (wallet, fragment_id). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors.
///
pub unsafe fn pending_transactions_delete(pending: PendingTransactionsPtr) {
    if !pending.is_null() {
        let boxed = Box::from_raw(pending);

        std::mem::drop(boxed);
    }
}

/// Get list of pending transaction id's
///
/// # Parameters
///
/// * wallet: the recovered wallet (see recover function);
/// * pending_transactions_out: an opaque type that works as an array
///
/// # Errors
///
/// * this function may fail if the wallet pointer is null;
/// * the block is not valid (cannot be decoded)
///
/// # Safety
///
/// This function dereference raw pointers (wallet). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors.
///
pub unsafe fn wallet_pending_transactions(
    wallet: WalletPtr,
    pending_transactions_out: *mut PendingTransactionsPtr,
) -> Result {
    let wallet = non_null!(wallet);

    let pending_transactions = PendingTransactions {
        fragment_ids: wallet
            .pending_transactions()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    };

    *pending_transactions_out =
        Box::into_raw(Box::new(pending_transactions)) as PendingTransactionsPtr;

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
/// This function dereference raw pointers (wallet, settings and conversion_out). Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
pub unsafe fn wallet_convert(
    wallet: WalletPtr,
    settings: SettingsPtr,
    conversion_out: *mut ConversionPtr,
) -> Result {
    let wallet: &mut Wallet = if let Some(wallet) = wallet.as_mut() {
        wallet
    } else {
        return Error::invalid_input("wallet").with(NulPtr).into();
    };
    let settings = if let Some(settings) = settings.as_ref() {
        settings.clone()
    } else {
        return Error::invalid_input("settings").with(NulPtr).into();
    };
    let conversion_out: &mut ConversionPtr = if let Some(conversion_out) = conversion_out.as_mut() {
        conversion_out
    } else {
        return Error::invalid_input("conversion_out").with(NulPtr).into();
    };

    let conversion = wallet.convert(settings);

    *conversion_out = Box::into_raw(Box::new(conversion));

    Result::success()
}

/// get the number of transactions built to convert the retrieved wallet
///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
pub unsafe fn wallet_convert_transactions_size(conversion: ConversionPtr) -> usize {
    conversion
        .as_ref()
        .map(|c| c.transactions.len())
        .unwrap_or_default()
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
pub unsafe fn wallet_convert_transactions_get(
    conversion: ConversionPtr,
    index: usize,
    transaction_out: *mut *const u8,
    transaction_size: *mut usize,
) -> Result {
    let conversion = if let Some(conversion) = conversion.as_ref() {
        conversion
    } else {
        return Error::invalid_input("conversion").with(NulPtr).into();
    };
    let transaction_out = if let Some(t) = transaction_out.as_mut() {
        t
    } else {
        return Error::invalid_input("transaction_out").with(NulPtr).into();
    };
    let transaction_size = if let Some(t) = transaction_size.as_mut() {
        t
    } else {
        return Error::invalid_input("transaction_size").with(NulPtr).into();
    };

    if let Some(t) = conversion.transactions.get(index) {
        *transaction_out = t.as_ptr();
        *transaction_size = t.len();
        Result::success()
    } else {
        Error::wallet_conversion().with(OutOfBound).into()
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
pub unsafe fn wallet_convert_ignored(
    conversion: ConversionPtr,
    value_out: *mut u64,
    ignored_out: *mut usize,
) -> Result {
    if let Some(c) = conversion.as_ref() {
        let v = *c
            .ignored
            .iter()
            .map(|i: &Input| i.value())
            .sum::<Value>()
            .as_ref();
        let l = c.ignored.len();

        if let Some(value_out) = value_out.as_mut() {
            *value_out = v
        }
        if let Some(ignored_out) = ignored_out.as_mut() {
            *ignored_out = l
        };

        Result::success()
    } else {
        Error::invalid_input("conversion").with(NulPtr).into()
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
pub fn wallet_set_state(wallet: WalletPtr, value: u64, counter: u32) -> Result {
    let wallet = if let Some(wallet) = unsafe { wallet.as_mut() } {
        wallet
    } else {
        return Error::invalid_input("wallet").with(NulPtr).into();
    };
    let value = Value(value);

    wallet.set_state(value, counter);

    Result::success()
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
    transaction_out: *mut *const u8,
    len_out: *mut usize,
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
    if len_out.is_null() {
        return Error::invalid_input("len_out").with(NulPtr).into();
    }

    let choice = Choice::new(choice);

    let transaction = match wallet.vote(settings, proposal, choice) {
        Ok(transaction) => Box::leak(transaction),
        Err(err) => return err.into(),
    };

    *transaction_out = transaction.as_ptr();
    *len_out = transaction.len();

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
pub fn wallet_delete_error(error: ErrorPtr) {
    if !error.is_null() {
        let boxed = unsafe { Box::from_raw(error) };

        std::mem::drop(boxed);
    }
}

/// delete the pointer and free the allocated memory
pub fn wallet_delete_settings(settings: SettingsPtr) {
    if !settings.is_null() {
        let boxed = unsafe { Box::from_raw(settings) };

        std::mem::drop(boxed);
    }
}

/// delete the pointer, zero all the keys and free the allocated memory
pub fn wallet_delete_wallet(wallet: WalletPtr) {
    if !wallet.is_null() {
        let boxed = unsafe { Box::from_raw(wallet) };

        std::mem::drop(boxed);
    }
}

/// delete the pointer
pub fn wallet_delete_conversion(conversion: ConversionPtr) {
    if !conversion.is_null() {
        let boxed = unsafe { Box::from_raw(conversion) };

        std::mem::drop(boxed);
    }
}

/// delete the pointer
pub fn wallet_delete_proposal(proposal: ProposalPtr) {
    if !proposal.is_null() {
        let boxed = unsafe { Box::from_raw(proposal) };

        std::mem::drop(boxed);
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
