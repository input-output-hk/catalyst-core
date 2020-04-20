use wallet::*;

const MNEMONICS: &str = "town lift more follow chronic lunch weird uniform earth census proof cave gap fancy topic year leader phrase state circle cloth reward dish survey act punch bounce";
const ADDRESS: &str = "DdzFFzCqrhtCvPjBLTJKJdNWzfhnJx3967QEcuhhm1PQ2ca13fNNMh5KZentH5aWLysjEBc1rKDYMS3noNKNyxdCL8NHUZznZj9gofQJ";

#[test]
fn restore_daedalus_paperwallet() {
    let address = ADDRESS.parse().unwrap();
    let wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("couldn't recover entropy")
        .build_daedalus()
        .expect("couldn't build daedalus");

    assert!(wallet.check_address(&address));
}
