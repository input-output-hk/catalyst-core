const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const WALLET_VALUE: u64 = 1000000 + 1 + 100;
use jormungandrwallet::*;

/// test to recover a daedalus style address in the test-vectors block0
///
#[ignore]
#[test]
fn daedalus1() {
    let mut wallet_ptr = std::ptr::null::<Wallet>() as WalletPtr;
    let mut settings_ptr = std::ptr::null::<Settings>() as SettingsPtr;
    let mut total_value = 0u64;

    let r = iohk_jormungandr_wallet_recover(
        "place help owner giggle record office lucky high canyon own spring cluster".as_ptr()
            as *const i8,
        std::ptr::null(),
        0,
        &mut wallet_ptr,
    );

    assert_eq!(
        RecoveringResult::Success,
        r,
        "expect to recover the wallet fully"
    );
    assert!(!wallet_ptr.is_null());

    let r = iohk_jormungandr_wallet_retrieve_funds(
        wallet_ptr,
        BLOCK0.as_ptr(),
        BLOCK0.len(),
        &mut settings_ptr,
    );

    assert_eq!(
        RecoveringResult::Success,
        r,
        "expect to recover the block0 fully"
    );
    assert!(!settings_ptr.is_null());

    let r = iohk_jormungandr_wallet_total_value(wallet_ptr, &mut total_value);

    assert_eq!(
        RecoveringResult::Success,
        r,
        "expect to get the total value"
    );
    assert_eq!(total_value, WALLET_VALUE);

    iohk_jormungandr_wallet_delete_settings(settings_ptr);
    iohk_jormungandr_wallet_delete_wallet(wallet_ptr);
}
