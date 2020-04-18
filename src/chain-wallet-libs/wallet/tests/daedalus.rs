const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
use chain_impl_mockchain::block::Block;
use chain_ser::mempack::{ReadBuf, Readable as _};
use wallet::*;

/// test to recover a daedalus style address in the test-vectors block0
///
#[test]
fn daedalus_wallet1() {
    const MNEMONICS: &str =
        "tired owner misery large dream glad upset welcome shuffle eagle pulp time";
    const WALLET_VALUE: u64 = 100000 + 1010;

    let mut wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics")
        .build_daedalus()
        .expect("recover a Legacy/Daedalus wallet");

    let mut block0_bytes = ReadBuf::from(BLOCK0);
    let block0 = Block::read(&mut block0_bytes).expect("valid block0");

    let _settings = wallet::Settings::new(&block0).expect("valid settings recovering");
    wallet.check_blocks(block0.contents.iter());

    let total_value = wallet.value_total();
    assert_eq!(total_value.as_ref(), &WALLET_VALUE);
}

/// test to recover a daedalus style address in the test-vectors block0
///
#[test]
fn daedalus_wallet2() {
    const MNEMONICS: &str = "edge club wrap where juice nephew whip entry cover bullet cause jeans";
    const WALLET_VALUE: u64 = 1000000 + 1 + 100;

    let mut wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics")
        .build_daedalus()
        .expect("recover a Legacy/Daedalus wallet");

    let mut block0_bytes = ReadBuf::from(BLOCK0);
    let block0 = Block::read(&mut block0_bytes).expect("valid block0");

    let _settings = wallet::Settings::new(&block0).expect("valid settings recovering");
    wallet.check_blocks(block0.contents.iter());

    let total_value = wallet.value_total();
    assert_eq!(total_value.as_ref(), &WALLET_VALUE);
}
