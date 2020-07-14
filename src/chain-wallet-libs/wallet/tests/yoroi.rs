mod utils;

use self::utils::State;
use chain_impl_mockchain::value::Value;
use wallet::{transaction::Dump, RecoveryBuilder};

const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const MNEMONICS: &str =
    "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone";
const WALLET_VALUE: Value = Value(1_000_000 + 10_000 + 10_000 + 1 + 100);

/// test to recover a daedalus style address in the test-vectors block0
///
#[test]
fn yoroi1() {
    let wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics");
    let mut yoroi = wallet
        .build_yoroi()
        .expect("recover an Icarus/Yoroi wallet");
    let account = wallet.build_wallet().expect("recover account");

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");
    let address = account.account_id().address(settings.discrimination());

    assert!(
        yoroi.check_fragments(state.initial_contents()),
        "failed to check fragments"
    );
    assert_eq!(yoroi.unconfirmed_value(), Some(WALLET_VALUE));

    let mut dump = Dump::new(settings, address);
    yoroi.dump_in(&mut dump);
    let (ignored, fragments) = dump.finalize();

    assert!(ignored.len() == 1, "there is only one ignored input");
    assert!(ignored[0].value() == Value(1), "the value ignored is `1`");

    state
        .apply_fragments(fragments.iter().map(|v| &v.1))
        .expect("the dump fragments should be valid");
}
