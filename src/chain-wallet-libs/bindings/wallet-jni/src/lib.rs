use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jbyte, jbyteArray, jint, jlong};
use jni::JNIEnv;
use std::convert::TryInto;
use std::ptr::{null, null_mut};
use wallet_core::c::*;
use wallet_core::c::{
    fragment::{fragment_from_raw, fragment_id},
    settings::{
        settings_block0_hash, settings_discrimination, settings_fees, settings_new, Discrimination,
        LinearFee, PerCertificateFee, PerVoteCertificateFee,
    },
};

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
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_importKeys(
    env: JNIEnv,
    _: JClass,
    account_key: jbyteArray,
    utxo_keys: jbyteArray,
) -> jlong {
    let account_key = {
        let len = env
            .get_array_length(account_key)
            .expect("Couldn't get account_key array length") as usize;

        if len != 64 {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "account key should have 64 bytes",
            );

            return 0;
        }

        let mut bytes = vec![0i8; len as usize];
        let _r = env.get_byte_array_region(account_key, 0, &mut bytes);
        bytes
    };

    let utxo_keys: Box<[i8]> = {
        let len = env
            .get_array_length(utxo_keys)
            .expect("Couldn't get account_key array length") as usize;

        let mut bytes = vec![0i8; len as usize];
        let _r = env.get_byte_array_region(utxo_keys, 0, &mut bytes);
        bytes.into_boxed_slice()
    };

    let mut wallet: WalletPtr = null_mut();
    let wallet_ptr: *mut WalletPtr = &mut wallet;

    let number_of_keys = match utxo_keys.len().checked_div(64) {
        Some(n) => n,
        None => {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "utxo_keys array is not a multiple of 64 bytes size",
            );

            return 0;
        }
    };

    let result = wallet_import_keys(
        account_key.as_ptr() as *const i8 as *const u8,
        utxo_keys.as_ptr() as *const i8 as *const [u8; 64],
        number_of_keys,
        wallet_ptr,
    );

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
pub extern "system" fn Java_com_iohk_jormungandrwallet_Settings_build(
    env: JNIEnv,
    _: JClass,
    linear_fees: JObject,
    discrimination: JObject,
    block0_hash: jbyteArray,
) -> jlong {
    let discrimination = match env.call_method(discrimination, "discriminant", "()B", &[]) {
        Ok(JValue::Byte(0)) => Discrimination::Production,
        Ok(JValue::Byte(1)) => Discrimination::Test,
        Err(_) => return std::ptr::null::<Settings>() as jlong,
        _ => unreachable!("there should be only two discrimination variants"),
    };

    let linear_fees = match env
        .call_method(linear_fees, "pack", "()[J", &[])
        .and_then(|packed| match packed {
            JValue::Object(array) => {
                let mut bytes = [0i64; 8];
                env.get_long_array_region(array.into_inner(), 0, &mut bytes)?;
                let bytes: [u64; 8] = unsafe { std::mem::transmute(bytes) };
                Ok(LinearFee {
                    constant: bytes[0],
                    coefficient: bytes[1],
                    certificate: bytes[2],
                    per_certificate_fees: PerCertificateFee {
                        certificate_pool_registration: bytes[3],
                        certificate_stake_delegation: bytes[4],
                        certificate_owner_stake_delegation: bytes[5],
                    },
                    per_vote_certificate_fees: PerVoteCertificateFee {
                        certificate_vote_plan: bytes[6],
                        certificate_vote_cast: bytes[7],
                    },
                })
            }
            _ => unreachable!("pack should return array"),
        }) {
        Ok(linear_fees) => linear_fees,
        Err(_) => return std::ptr::null::<Settings>() as jlong,
    };

    let len = env
        .get_array_length(block0_hash)
        .expect("Couldn't get block0 array length") as usize;

    // TODO: define a constant HEADER_ID_SIZE somewhere
    assert_eq!(len, 32);

    // TODO: define a constant HEADER_ID_SIZE somewhere
    let mut bytes = [0i8; 32];
    let _r = env.get_byte_array_region(block0_hash, 0, &mut bytes);

    let mut settings_out = std::ptr::null_mut();

    let result = unsafe {
        settings_new(
            linear_fees,
            discrimination,
            bytes.as_mut_ptr() as *mut u8,
            &mut settings_out as *mut *mut Settings,
        )
    };

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
    }

    settings_out as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Settings_fees(
    env: JNIEnv,
    _: JClass,
    settings: jlong,
) -> jni::sys::jobject {
    let settings_ptr: SettingsPtr = settings as SettingsPtr;

    let mut linear_fee = LinearFee::default();

    let result = unsafe { settings_fees(settings_ptr, &mut linear_fee as *mut LinearFee) };

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
        std::ptr::null_mut::<jni::sys::_jobject>()
    } else {
        match env
            .find_class("com/iohk/jormungandrwallet/Settings$LinearFees")
            .and_then(|class_id| {
                let linear_fees = env.new_object(
                    class_id,
                    concat! {
                        "(",
                        "JJJ", // the ones in the top level
                        "JJJ", // per certificate
                        "JJ",  // per vote certificate
                        ")V",
                    },
                    &[
                        JValue::Long(linear_fee.constant as i64),
                        JValue::Long(linear_fee.coefficient as i64),
                        JValue::Long(linear_fee.certificate as i64),
                        JValue::Long(
                            linear_fee
                                .per_certificate_fees
                                .certificate_pool_registration as i64,
                        ),
                        JValue::Long(
                            linear_fee.per_certificate_fees.certificate_stake_delegation as i64,
                        ),
                        JValue::Long(
                            linear_fee
                                .per_certificate_fees
                                .certificate_owner_stake_delegation
                                as i64,
                        ),
                        JValue::Long(
                            linear_fee.per_vote_certificate_fees.certificate_vote_plan as i64,
                        ),
                        JValue::Long(
                            linear_fee.per_vote_certificate_fees.certificate_vote_cast as i64,
                        ),
                    ],
                )?;

                Ok(linear_fees.into_inner())
            }) {
            Ok(linear_fees) => linear_fees,
            Err(_) => std::ptr::null_mut::<jni::sys::_jobject>(),
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Settings_discrimination(
    env: JNIEnv,
    _: JClass,
    settings: jlong,
) -> jni::sys::jobject {
    let settings_ptr: SettingsPtr = settings as SettingsPtr;

    // it doesn't matter which variant we use here, it's going to be replaced by
    // the function if necessary
    let mut discrimination = Discrimination::Production;

    let result = unsafe {
        settings_discrimination(settings_ptr, &mut discrimination as *mut Discrimination)
    };

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
        std::ptr::null_mut::<jni::sys::_jobject>()
    } else {
        let field = match discrimination {
            Discrimination::Production => "PRODUCTION",
            Discrimination::Test => "TEST",
        };
        match env.get_static_field(
            "com/iohk/jormungandrwallet/Settings$Discrimination",
            field,
            "Lcom/iohk/jormungandrwallet/Settings$Discrimination;",
        ) {
            Ok(JValue::Object(v)) => v.into_inner(),
            Ok(_) => unreachable!("Discrimination is an enum"),
            Err(_) => std::ptr::null_mut::<jni::sys::_jobject>(),
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Settings_block0Hash(
    env: JNIEnv,
    _: JClass,
    settings: jlong,
) -> jbyteArray {
    let settings_ptr: SettingsPtr = settings as SettingsPtr;

    // TODO: define a constant HEADER_ID_SIZE somewhere
    let mut block0_hash = [0i8; 32];

    let result = unsafe { settings_block0_hash(settings_ptr, block0_hash.as_mut_ptr() as *mut u8) };

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
        std::ptr::null_mut::<jni::sys::_jobject>()
    } else {
        let array = env
            .new_byte_array(block0_hash.len() as i32)
            .expect("Failed to create new byte array");

        env.set_byte_array_region(array, 0, &block0_hash)
            .expect("Couldn't copy array to jvm");

        array
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
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_spendingCounter(
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
) -> jlong {
    let wallet_ptr: WalletPtr = wallet as WalletPtr;
    let mut value: u32 = 0;
    let result = wallet_spending_counter(wallet_ptr, &mut value as *mut u32);

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
    }

    value as jlong
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_id(
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
) -> jbyteArray {
    let wallet_ptr = wallet as WalletPtr;

    let array = env
        .new_byte_array(32)
        .expect("Failed to create new byte array");

    let mut id_out = [0i8; 32];

    let result = wallet_id(wallet_ptr, id_out.as_mut_ptr() as *mut u8);

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
    } else {
        env.set_byte_array_region(array, 0, &id_out)
            .expect("Couldn't copy array to jvm");
    }

    array
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_pendingTransactions(
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
) -> jlong {
    let wallet_ptr: WalletPtr = wallet as WalletPtr;

    let mut pending_transactions = null_mut();

    if !wallet_ptr.is_null() {
        let result = wallet_pending_transactions(wallet_ptr, &mut pending_transactions);

        if let Some(error) = result.error() {
            let _ = env.throw(error.to_string());
        }
    }

    pending_transactions as jlong
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_confirmTransaction(
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
    fragment_id: jbyteArray,
) {
    let wallet_ptr: WalletPtr = wallet as WalletPtr;
    let len = env
        .get_array_length(fragment_id)
        .expect("Couldn't get block0 array length") as usize;

    debug_assert_eq!(len, wallet_core::c::FRAGMENT_ID_LENGTH);

    let mut bytes = vec![0i8; len as usize];

    let _r = env.get_byte_array_region(fragment_id, 0, &mut bytes);

    if !wallet_ptr.is_null() {
        let result = wallet_confirm_transaction(wallet_ptr, bytes.as_ptr() as *const u8);
        if let Some(error) = result.error() {
            let _ = env.throw(error.to_string());
        }
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
    conversion: jlong,
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
    conversion: jlong,
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

/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
/// The `callback` parameter it's expected to be a java object with a `call` method that takes 2 parameters
///
/// long value: will returns the total value lost into dust inputs
/// long ignored: will returns the number of dust utxos
#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Conversion_ignored(
    env: JNIEnv,
    _: JClass,
    conversion: jlong,
    callback: JObject,
) {
    let conversion = conversion as ConversionPtr;
    let mut value_out: u64 = 0;
    let mut ignored_out: usize = 0;

    let result = unsafe { wallet_convert_ignored(conversion, &mut value_out, &mut ignored_out) };

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
    }

    let result = env.call_method(
        callback,
        "call",
        "(JJ)V",
        &[
            JValue::Long(value_out as jlong),
            JValue::Long(ignored_out as jlong),
        ],
    );

    // throw error as exception only the call didn't already threw an error
    // if this happens then there is nothing to do, but it's a bit more gentle than a panic
    // this can happen for example if the type signature of the callback is invalid
    if let (Err(error), false) = (
        result,
        env.exception_check()
            .expect("error checking if exception was thrown"),
    ) {
        let _ = env.throw(error.to_string());
    }
}

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
    wallet: jlong,
    value: jlong,
    counter: jlong,
) {
    let wallet = wallet as WalletPtr;
    let r = wallet_set_state(wallet, value as u64, counter as u32);

    if let Some(error) = r.error() {
        let _ = env.throw(error.to_string());
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Proposal_withPublicPayload(
    env: JNIEnv,
    _: JClass,
    vote_plan_id: jbyteArray,
    index: jint,
    num_choices: jint,
) -> jlong {
    let size = env.get_array_length(vote_plan_id).expect("invalid array");

    let mut buffer = vec![0i8; size as usize];
    env.get_byte_array_region(vote_plan_id, 0, &mut buffer)
        .expect("invalid byte arrray read");

    let index: u8 = match index.try_into() {
        Ok(index) => index,
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "index should be a number between 0 and 255",
            );

            return 0;
        }
    };

    let num_choices: u8 = match num_choices.try_into() {
        Ok(index) => index,
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "numChoices should be a number between 0 and 255",
            );

            return 0;
        }
    };

    let mut proposal = null_mut();
    let r = unsafe {
        vote::proposal_new(
            buffer.as_ptr() as *const u8,
            index,
            num_choices,
            vote::ProposalPublic,
            &mut proposal,
        )
    };

    if let Some(error) = r.error() {
        let _ = env.throw(error.to_string());
    }

    proposal as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Proposal_withPrivatePayload(
    env: JNIEnv,
    _: JClass,
    vote_plan_id: jbyteArray,
    index: jint,
    num_choices: jint,
    encryption_key: JString,
) -> jlong {
    let id_size = env.get_array_length(vote_plan_id).expect("invalid array");

    let mut buffer = vec![0i8; id_size as usize];
    env.get_byte_array_region(vote_plan_id, 0, &mut buffer)
        .expect("invalid byte arrray read");

    let encryption_key = env
        .get_string(encryption_key)
        .expect("Couldn't get bech32 string");

    let index: u8 = match index.try_into() {
        Ok(index) => index,
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "index should be a number between 0 and 255",
            );

            return 0;
        }
    };

    let num_choices: u8 = match num_choices.try_into() {
        Ok(index) => index,
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "numChoices should be a number between 0 and 255",
            );

            return 0;
        }
    };

    let mut proposal = null_mut();
    let r = unsafe {
        vote::proposal_new(
            buffer.as_ptr() as *const u8,
            index,
            num_choices,
            vote::ProposalPrivate(&encryption_key),
            &mut proposal,
        )
    };

    if let Some(error) = r.error() {
        let _ = env.throw(error.to_string());
    }

    proposal as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Proposal_delete(
    _: JNIEnv,
    _: JClass,
    proposal: jlong,
) {
    let proposal = proposal as ProposalPtr;
    if !proposal.is_null() {
        wallet_delete_proposal(proposal);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_Wallet_voteCast(
    env: JNIEnv,
    _: JClass,
    wallet: jlong,
    settings: jlong,
    proposal: jlong,
    choice: jint,
) -> jbyteArray {
    let wallet_ptr = wallet as WalletPtr;
    let settings_ptr = settings as SettingsPtr;
    let proposal_ptr = proposal as ProposalPtr;

    let choice: u8 = match choice.try_into() {
        Ok(index) => index,
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "choice should be a number between 0 and 255",
            );

            return null_mut() as jbyteArray;
        }
    };

    let mut transaction_out: *const u8 = null();
    let mut transaction_size: usize = 0;

    let r = unsafe {
        wallet_vote_cast(
            wallet_ptr,
            settings_ptr,
            proposal_ptr,
            choice,
            &mut transaction_out as *mut *const u8,
            &mut transaction_size as *mut usize,
        )
    };

    if let Some(error) = r.error() {
        let _ = env.throw(error.to_string());
        return null_mut() as jbyteArray;
    }

    let array = env
        .new_byte_array(transaction_size as jint)
        .expect("Failed to create new byte array");

    debug_assert!(!transaction_out.is_null());
    let slice =
        unsafe { std::slice::from_raw_parts(transaction_out as *const jbyte, transaction_size) };

    env.set_byte_array_region(array, 0, slice)
        .expect("Couldn't copy array to jvm");

    // wallet_vote_cast leaks the buffer, so we need to deallocate that memory,
    // set_byte_array_region does a *copy* of the buffer so we don't need it anymore.
    unsafe { Box::from_raw(slice.as_ptr() as *mut u8) };

    array
}

/// # TODO
#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_PendingTransactions_len(
    env: JNIEnv,
    _: JClass,
    pending: jlong,
) -> jint {
    let pending = pending as PendingTransactionsPtr;
    let mut len: usize = 0;
    if !pending.is_null() {
        let r = unsafe { pending_transactions_len(pending, &mut len) };

        if let Some(error) = r.error() {
            let _ = env.throw(error.to_string());
        }
    }

    len as jint
}

// # TODO (doc)
#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_PendingTransactions_get(
    env: JNIEnv,
    _: JClass,
    pending: jlong,
    index: jint,
) -> jbyteArray {
    let pending = pending as PendingTransactionsPtr;

    if index.is_negative() {
        let _ = env.throw_new(
            "java/lang/IndexOutOfBoundsException",
            "Conversion transaction index should be a positive number",
        );
        return null_mut();
    }

    let index = index as usize;
    let mut id_out: *const u8 = null();
    let id_size: usize = FRAGMENT_ID_LENGTH;

    let result =
        unsafe { pending_transactions_get(pending, index, (&mut id_out) as *mut *const u8) };

    match result.error() {
        None => {
            let slice = unsafe { std::slice::from_raw_parts(id_out as *const jbyte, id_size) };

            let array = env
                .new_byte_array(id_size as jint)
                .expect("Failed to create new byte array");

            env.set_byte_array_region(array, 0, slice)
                .expect("Couldn't copy array to jvm");

            array
        }
        Some(error) => {
            let _ = env.throw(error.to_string());
            null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_PendingTransactions_delete(
    _env: JNIEnv,
    _: JClass,
    pending: jlong,
) {
    let pending = pending as PendingTransactionsPtr;
    if !pending.is_null() {
        unsafe { pending_transactions_delete(pending) };
    }
}

#[no_mangle]
pub extern "system" fn Java_com_iohk_jormungandrwallet_SymmetricCipher_decrypt(
    env: JNIEnv,
    _: JClass,
    password: jbyteArray,
    ciphertext: jbyteArray,
) -> jbyteArray {
    let password = {
        let size = env.get_array_length(password).expect("invalid array");
        let mut buffer = vec![0i8; size as usize];
        env.get_byte_array_region(password, 0, &mut buffer)
            .expect("invalid byte arrray read");
        buffer
    };

    let ciphertext = {
        let size = env.get_array_length(ciphertext).expect("invalid array");
        let mut buffer = vec![0i8; size as usize];
        env.get_byte_array_region(ciphertext, 0, &mut buffer)
            .expect("invalid byte arrray read");

        buffer
    };

    let mut plaintext_out: *const u8 = null_mut();
    let mut plaintext_out_length = 0usize;
    let result = unsafe {
        symmetric_cipher_decrypt(
            password.as_ptr() as *const u8,
            password.len(),
            ciphertext.as_ptr() as *const u8,
            ciphertext.len(),
            (&mut plaintext_out) as *mut *const u8,
            (&mut plaintext_out_length) as *mut usize,
        )
    };

    match result.error() {
        None => {
            let slice = unsafe {
                std::slice::from_raw_parts(plaintext_out as *const jbyte, plaintext_out_length)
            };

            let array = env
                .new_byte_array(plaintext_out_length as jint)
                .expect("Failed to create new byte array");

            env.set_byte_array_region(array, 0, slice)
                .expect("Couldn't copy array to jvm");

            unsafe {
                delete_buffer(plaintext_out as *mut u8, plaintext_out_length);
            }

            array
        }
        Some(error) => {
            let _ = env.throw(error.to_string());
            null_mut()
        }
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
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Fragment_fromBytes(
    env: JNIEnv,
    _: JClass,
    buffer: jbyteArray,
) -> jlong {
    let len = env
        .get_array_length(buffer)
        .expect("Couldn't get block0 array length") as usize;

    let mut bytes = vec![0i8; len as usize];

    env.get_byte_array_region(buffer, 0, &mut bytes).unwrap();

    let mut ptr: FragmentPtr = null_mut();

    let result = fragment_from_raw(
        bytes.as_ptr().cast::<u8>(),
        len,
        &mut ptr as *mut FragmentPtr,
    );

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
    }

    ptr as jlong
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Fragment_id(
    env: JNIEnv,
    _: JClass,
    fragment: jlong,
) -> jbyteArray {
    let mut id = [0u8; FRAGMENT_ID_LENGTH];

    let result = fragment_id(fragment as FragmentPtr, id.as_mut_ptr());

    let array = env
        .new_byte_array(id.len() as jint)
        .expect("Failed to create new byte array");

    match result.error() {
        None => {
            let slice = std::slice::from_raw_parts(id.as_ptr() as *const jbyte, id.len());

            env.set_byte_array_region(array, 0, slice)
                .expect("Couldn't copy array to jvm");
        }
        Some(error) => {
            let _ = env.throw(error.to_string());
        }
    };

    array
}

///
/// # Safety
///
/// This function dereference raw pointers. Even though
/// the function checks if the pointers are null. Mind not to put random values
/// in or you may see unexpected behaviors
///
#[no_mangle]
pub unsafe extern "system" fn Java_com_iohk_jormungandrwallet_Fragment_delete(
    _env: JNIEnv,
    _: JClass,
    fragment: jlong,
) {
    fragment::fragment_delete(fragment as FragmentPtr);
}
