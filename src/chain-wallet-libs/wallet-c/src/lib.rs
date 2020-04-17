use chain_impl_mockchain::{
    block::Block,
    transaction::{Input, NoExtra, Transaction},
    value::Value,
};
use chain_ser::mempack::{ReadBuf, Readable as _};
use std::{ffi::CStr, os::raw::c_char};

/// the wallet
///
/// * use the `recover` function to recover the wallet from the mnemonics/password;
/// * use the `retrieve_funds` to retrieve initial funds (if necessary) from the block0;
///   then you can use `total_value` to see how much was recovered from the initial block0;
///
/// DO NOT FORGET:
///
/// to delete with `delete_wallet` once you do not need the wallet as there are
/// cryptographic material and forgetting to delete it properly will result
/// in risking leaking your wallet private keys
///
pub struct Wallet {
    account: wallet::Wallet,
    daedalus: wallet::RecoveringDaedalus,
    icarus: wallet::RecoveringIcarus,
}

/// the blockchain settings
///
/// this can be retrieved when parsing the block0
///
/// It contains all the necessary information to make valid transactions
/// (including transferring legacy wallets into a new secure wallet).
pub struct Settings(wallet::Settings);

pub struct Conversion {
    ignored: Vec<Input>,
    transactions: Vec<Transaction<NoExtra>>,
}

pub type WalletPtr = *mut Wallet;
pub type SettingsPtr = *mut Settings;
pub type ConversionPtr = *mut Conversion;

/// result error code
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum RecoveringResult {
    /// returned if the function succeed
    Success = 0,
    /// this error is returned if the users mnemonics are invalid
    InvalidMnemonics,
    /// happens if the block is not valid
    InvalidBlockFormat,
    /// index is out of bound
    IndexOutOfBound,
    /// a pointer was null where it was expected it to be non null
    PtrIsNull,
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
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_recover(
    mnemonics: *const c_char,
    password: *const u8,
    password_length: usize,
    wallet_out: *mut WalletPtr,
) -> RecoveringResult {
    let wallet_out: &mut WalletPtr = if let Some(wallet_out) = unsafe { wallet_out.as_mut() } {
        wallet_out
    } else {
        return RecoveringResult::PtrIsNull;
    };
    let mnemonics = unsafe { CStr::from_ptr(mnemonics) };

    let mnemonics = mnemonics.to_string_lossy();

    let builder = wallet::RecoveryBuilder::new();

    let paperwallet: Option<(bip39::Entropy, &str)> = match wallet::get_scrambled_input(&mnemonics)
    {
        Ok(paperwallet_builder) => paperwallet_builder,
        Err(_) => return RecoveringResult::InvalidMnemonics,
    };

    let builder = match paperwallet {
        Some((entropy, pass)) => match builder.paperwallet(pass, entropy) {
            Ok(builder) => builder,
            Err(_) => return RecoveringResult::InvalidMnemonics,
        },
        None => {
            if !password.is_null() && password_length == 0 {
                match builder.mnemonics(&bip39::dictionary::ENGLISH, mnemonics) {
                    Ok(builder) => builder,
                    Err(_) => return RecoveringResult::InvalidMnemonics,
                }
            } else {
                todo!()
            }
        }
    };

    // calling this function cannot fail, we already
    // have the mnemonics set in the builder, and there is no password set
    let daedalus = builder
        .build_daedalus()
        .expect("build the daedalus wallet as expected");
    // calling this function cannot fail, we already
    // have the mnemonics set in the builder, and there is no password set
    let icarus = builder
        .build_yoroi()
        .expect("build the daedalus wallet as expected");
    // calling this function cannot fail as we have set the mnemonics already
    // and no password is valid (though it is weak security from daedalus wallet PoV)
    let account = builder
        .build_wallet()
        .expect("build the account cannot fail as expected");

    let recovering = Wallet {
        account,
        daedalus,
        icarus,
    };

    *wallet_out = Box::into_raw(Box::new(recovering));
    RecoveringResult::Success
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
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_retrieve_funds(
    wallet: WalletPtr,
    block0: *const u8,
    block0_length: usize,
    settings_out: *mut SettingsPtr,
) -> RecoveringResult {
    let wallet: &mut Wallet = if let Some(wallet) = unsafe { wallet.as_mut() } {
        wallet
    } else {
        return RecoveringResult::PtrIsNull;
    };
    let settings_out: &mut SettingsPtr =
        if let Some(settings_out) = unsafe { settings_out.as_mut() } {
            settings_out
        } else {
            return RecoveringResult::PtrIsNull;
        };

    let block0_bytes = unsafe { std::slice::from_raw_parts(block0, block0_length) };

    let mut block0_bytes = ReadBuf::from(block0_bytes);
    let block0 = if let Ok(block) = Block::read(&mut block0_bytes) {
        block
    } else {
        return RecoveringResult::InvalidBlockFormat;
    };

    let settings = Settings(wallet::Settings::new(&block0).unwrap());
    wallet.daedalus.check_blocks(block0.contents.iter());
    wallet.icarus.check_blocks(block0.contents.iter());

    *settings_out = Box::into_raw(Box::new(settings));

    RecoveringResult::Success
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
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_convert(
    wallet: WalletPtr,
    settings: SettingsPtr,
    conversion_out: *mut ConversionPtr,
) -> RecoveringResult {
    let wallet: &mut Wallet = if let Some(wallet) = unsafe { wallet.as_mut() } {
        wallet
    } else {
        return RecoveringResult::PtrIsNull;
    };
    let settings = if let Some(settings) = unsafe { settings.as_ref() } {
        settings.0.clone()
    } else {
        return RecoveringResult::PtrIsNull;
    };
    let conversion_out: &mut ConversionPtr =
        if let Some(conversion_out) = unsafe { conversion_out.as_mut() } {
            conversion_out
        } else {
            return RecoveringResult::PtrIsNull;
        };

    let address = wallet
        .account
        .account_id()
        .address(chain_addr::Discrimination::Production);

    let mut dump = wallet::transaction::Dump::new(settings, address);

    wallet.daedalus.dump_in(&mut dump);
    wallet.icarus.dump_in(&mut dump);

    let (ignored, transactions) = dump.finalize();

    let conversion = Conversion {
        ignored,
        transactions,
    };

    *conversion_out = Box::into_raw(Box::new(conversion));

    RecoveringResult::Success
}

/// get the number of transactions built to convert the retrieved wallet
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_convert_transactions_size(
    conversion: ConversionPtr,
) -> usize {
    unsafe {
        conversion
            .as_ref()
            .map(|c| c.transactions.len())
            .unwrap_or_default()
    }
}

/// retrieve the index-nth transactions in the conversions starting from 0
/// and finishing at `size-1` where size is retrieved from
/// `iohk_jormungandr_wallet_convert_transactions_size`.
///
/// the memory allocated returned is not owned and should not be kept
/// for longer than potential call to `iohk_jormungandr_wallet_delete_conversion`
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_convert_transactions_get(
    conversion: ConversionPtr,
    index: usize,
    transaction_out: *mut *const u8,
    transaction_size: *mut usize,
) -> RecoveringResult {
    let conversion = if let Some(conversion) = unsafe { conversion.as_ref() } {
        conversion
    } else {
        return RecoveringResult::PtrIsNull;
    };
    let transaction_out = if let Some(t) = unsafe { transaction_out.as_mut() } {
        t
    } else {
        return RecoveringResult::PtrIsNull;
    };
    let transaction_size = if let Some(t) = unsafe { transaction_size.as_mut() } {
        t
    } else {
        return RecoveringResult::PtrIsNull;
    };

    if let Some(t) = conversion.transactions.get(index) {
        *transaction_out = t.as_ref().as_ptr();
        *transaction_size = t.as_ref().len();
        RecoveringResult::Success
    } else {
        RecoveringResult::IndexOutOfBound
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
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_convert_ignored(
    conversion: ConversionPtr,
    value_out: *mut u64,
    ignored_out: *mut usize,
) -> RecoveringResult {
    if let Some(c) = unsafe { conversion.as_ref() } {
        let v = *c
            .ignored
            .iter()
            .map(|i: &Input| i.value())
            .sum::<Value>()
            .as_ref();
        let l = c.ignored.len();

        if let Some(value_out) = unsafe { value_out.as_mut() } {
            *value_out = v
        }
        if let Some(ignored_out) = unsafe { ignored_out.as_mut() } {
            *ignored_out = l
        };

        RecoveringResult::Success
    } else {
        RecoveringResult::PtrIsNull
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
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_total_value(
    wallet: WalletPtr,
    total_out: *mut u64,
) -> RecoveringResult {
    let wallet: &Wallet = if let Some(wallet) = unsafe { wallet.as_mut() } {
        wallet
    } else {
        return RecoveringResult::PtrIsNull;
    };

    if let Some(total_out) = unsafe { total_out.as_mut() } {
        let total = wallet
            .icarus
            .value_total()
            .saturating_add(wallet.daedalus.value_total());

        *total_out = *total.as_ref();
    }

    RecoveringResult::Success
}

/// delete the pointer and free the allocated memory
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_settings(settings: SettingsPtr) {
    if !settings.is_null() {
        let boxed = unsafe { Box::from_raw(settings) };

        std::mem::drop(boxed);
    }
}

/// delete the pointer, zero all the keys and free the allocated memory
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_wallet(wallet: WalletPtr) {
    if !wallet.is_null() {
        let boxed = unsafe { Box::from_raw(wallet) };

        std::mem::drop(boxed);
    }
}

/// delete the pointer
#[no_mangle]
pub extern "C" fn iohk_jormungandr_wallet_delete_conversion(conversion: ConversionPtr) {
    if !conversion.is_null() {
        let boxed = unsafe { Box::from_raw(conversion) };

        std::mem::drop(boxed);
    }
}
