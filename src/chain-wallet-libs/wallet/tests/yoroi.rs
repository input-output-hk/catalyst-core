const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const MNEMONICS: &str =
    "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone";
const WALLET_VALUE: u64 = 1_000_000 + 10_000 + 10_000 + 1 + 100;
use chain_impl_mockchain::block::Block;
use chain_ser::mempack::{ReadBuf, Readable as _};
use wallet::*;

/// test to recover a daedalus style address in the test-vectors block0
///
#[test]
fn yoroi1() {
    let mut wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics")
        .build_yoroi()
        .expect("recover an Icarus/Yoroi wallet");

    let mut block0_bytes = ReadBuf::from(BLOCK0);
    let block0 = Block::read(&mut block0_bytes).expect("valid block0");

    let _settings = wallet::Settings::new(&block0).expect("valid settings recovering");
    wallet.check_fragments(block0.contents.iter());

    let total_value = wallet.value_total();
    assert_eq!(total_value.as_ref(), &WALLET_VALUE);
}
