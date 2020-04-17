use bip39;
use cardano_legacy_address;
use chain_impl_mockchain::legacy::OldAddress;
use wallet::*;

use std::str::FromStr;
const MNEMONICS: &'static str = "town lift more follow chronic lunch weird uniform earth census proof cave gap fancy topic year leader phrase state circle cloth reward dish survey act punch bounce";
#[test]
fn restore_daedalus_paperwallet() {
    let (entropy, pass) = get_scrambled_input(MNEMONICS)
        .expect("invalid mnemonics")
        .expect("words should be 27");

    let expected_address = "DdzFFzCqrhtCvPjBLTJKJdNWzfhnJx3967QEcuhhm1PQ2ca13fNNMh5KZentH5aWLysjEBc1rKDYMS3noNKNyxdCL8NHUZznZj9gofQJ";

    let recovering_daedalus = RecoveryBuilder::new()
        .paperwallet(pass, entropy)
        .expect("couldn't recover entropy")
        .build_daedalus()
        .expect("couldn't build daedalus");

    // TODO: assert something? But I still don't know how
    assert!(false);
}
