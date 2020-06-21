const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");

mod utils;

use self::utils::State;
use chain_impl_mockchain::value::Value;
use wallet::{transaction::Dump, RecoveryBuilder};

/// test to recover a daedalus style address in the test-vectors block0
///
#[test]
fn daedalus_wallet1() {
    const MNEMONICS: &str =
        "tired owner misery large dream glad upset welcome shuffle eagle pulp time";
    const WALLET_VALUE: u64 = 100_000 + 1010;

    let wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics");
    let mut daedalus = wallet
        .build_daedalus()
        .expect("recover a Legacy/Daedalus wallet");
    let account = wallet.build_wallet().expect("recover account");

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");
    let address = account.account_id().address(settings.discrimination());

    daedalus
        .check_fragments(state.initial_contents())
        .expect("failed to check fragments");

    assert_eq!(daedalus.value_total().as_ref(), &WALLET_VALUE);

    let mut dump = Dump::new(settings, address);
    daedalus.dump_in(&mut dump);
    let (ignored, fragments) = dump.finalize();

    assert!(ignored.is_empty());

    state
        .apply_fragments(fragments.iter().map(|v| &v.1))
        .expect("the dump fragments should be valid");
}

/// test to recover a daedalus style address in the test-vectors block0
///
#[test]
fn daedalus_wallet2() {
    const MNEMONICS: &str = "edge club wrap where juice nephew whip entry cover bullet cause jeans";
    const WALLET_VALUE: u64 = 1_000_000 + 1 + 100;

    let wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics");
    let mut daedalus = wallet
        .build_daedalus()
        .expect("recover a Legacy/Daedalus wallet");
    let account = wallet.build_wallet().expect("recover account");

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");
    let address = account.account_id().address(settings.discrimination());

    daedalus
        .check_fragments(state.initial_contents())
        .expect("failed to check fragments");

    assert_eq!(daedalus.value_total().as_ref(), &WALLET_VALUE);

    let mut dump = Dump::new(settings, address);
    daedalus.dump_in(&mut dump);
    let (ignored, fragments) = dump.finalize();

    assert!(ignored.len() == 1, "there is only one ignored input");
    assert!(ignored[0].value() == Value(1), "the value ignored is `1`");

    state
        .apply_fragments(fragments.iter().map(|v| &v.1))
        .expect("the dump fragments should be valid");
}
