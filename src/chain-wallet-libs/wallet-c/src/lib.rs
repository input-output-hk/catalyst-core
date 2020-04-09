use std::{
    ffi::CStr,
    os::raw::c_char,
};
use chain_impl_mockchain::block::Block;
use chain_ser::mempack::{ReadBuf, Readable as _};

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

type WalletPtr = *mut Wallet;
type SettingsPtr = *mut Settings;

/// result error code
#[repr(u8)]
pub enum RecoveringResult {
    /// returned if the function succeed
    Success = 0,
    /// this error is returned if the users mnemonics are invalid
    InvalidMnemonics,
    /// happens if the block is not valid
    InvalidBlockFormat,
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
/// * protocol_magic: the legacy cardano haskell protocol magic of the blockchain, mandatory
///   for Icarus (yoroi) wallets;
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
    protocol_magic: u32,
    wallet_out: *mut WalletPtr,
) -> RecoveringResult
{
    let wallet_out: &mut WalletPtr = if let Some(wallet_out) = unsafe { wallet_out.as_mut() } {
        wallet_out
    } else {
        return RecoveringResult::PtrIsNull
    };
    let mnemonics = unsafe { CStr::from_ptr(mnemonics) };

    let mnemonics = mnemonics.to_string_lossy();

    let builder = wallet::RecoveryBuilder::new();
    let builder = builder.protocol_magic(protocol_magic);
    let builder = if let Ok(builder) = builder.mnemonics(&bip39::dictionary::ENGLISH, mnemonics) {
        builder
    } else {
        return RecoveringResult::InvalidMnemonics;
    };
    let builder = if !password.is_null() && password_length > 0 {
        todo!()
    } else {
        builder
    };

    // calling this function cannot fail, we already
    // have the mnemonics set in the builder, and there is no password set
    let daedalus= builder.build_daedalus()
        .expect("build the daedalus wallet as expected");
    // calling this function cannot fail, we already
    // have the mnemonics set in the builder, and there is no password set
    let icarus = builder.build_yoroi()
        .expect("build the daedalus wallet as expected");
    // calling this function cannot fail as we have set the mnemonics already
    // and no password is valid (though it is weak security from daedalus wallet PoV)
    let account = builder.build_wallet()
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
    let wallet : &mut Wallet = if let Some(wallet) = unsafe { wallet.as_mut() } {
        wallet
    } else {
        return RecoveringResult::PtrIsNull
    };
    let settings_out : &mut SettingsPtr = if let Some(settings_out) = unsafe { settings_out.as_mut() } {
        settings_out
    } else {
        return RecoveringResult::PtrIsNull
    };

    let block0_bytes = unsafe { std::slice::from_raw_parts(block0, block0_length) };

    let mut block0_bytes = ReadBuf::from(block0_bytes);
    let block0 = if let Ok(block) = Block::read(&mut block0_bytes) {
        block
    } else {
        return RecoveringResult::InvalidBlockFormat
    };

    let settings = Settings(wallet::Settings::new(&block0).unwrap());
    wallet.daedalus.check_blocks(block0.contents.iter());
    wallet.icarus.check_blocks(block0.contents.iter());

    *settings_out = Box::into_raw(Box::new(settings));

    RecoveringResult::Success
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
pub extern "C" fn iohk_jormungandr_wallet_total_value(wallet: WalletPtr, total_out: *mut u64) -> RecoveringResult
{
    let wallet : &Wallet = if let Some(wallet) = unsafe { wallet.as_mut() } {
        wallet
    } else {
        return RecoveringResult::PtrIsNull
    };

    if let Some(total_out) = unsafe { total_out.as_mut() } {
        let total = wallet.icarus.value_total().saturating_add(wallet.daedalus.value_total());

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