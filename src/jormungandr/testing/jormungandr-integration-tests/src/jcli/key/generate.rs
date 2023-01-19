use jormungandr_automation::jcli::JCli;

#[test]
pub fn test_ed25519_key_generation() {
    let jcli: JCli = Default::default();
    let generated_key = jcli.key().generate("ed25519");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_ed25510bip32_key_generation() {
    let jcli: JCli = Default::default();
    let generated_key = jcli.key().generate("ed25519-bip32");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_ed25519extended_key_generation() {
    let jcli: JCli = Default::default();
    let generated_key = jcli.key().generate("ed25519-extended");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_curve25519_2hashdh_key_generation() {
    let jcli: JCli = Default::default();
    let generated_key = jcli.key().generate("ristretto-group2-hash-dh");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_sumed25519_12_key_generation() {
    let jcli: JCli = Default::default();
    let generated_key = jcli.key().generate("sum-ed25519-12");
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_unknown_key_type_generation() {
    let jcli: JCli = Default::default();
    jcli.key().generate_expect_fail(
        "unknown",
        "'unknown' isn't a valid value for '--type <KEY_TYPE>'",
    );
}

#[test]
pub fn test_key_with_seed_generation() {
    let jcli: JCli = Default::default();
    let correct_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada3e135";
    let generated_key = jcli
        .key()
        .generate_with_seed("ed25519-extended", correct_seed);
    assert_ne!(generated_key, "", "generated key is empty");
}

#[test]
pub fn test_key_with_too_short_seed_generation() {
    let too_short_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcadaa";
    test_key_invalid_seed_length(too_short_seed);
}

#[test]
pub fn test_key_with_too_long_seed_generation() {
    let too_long_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada0234212";
    test_key_invalid_seed_length(too_long_seed);
}

fn test_key_invalid_seed_length(seed: &str) {
    let jcli: JCli = Default::default();
    jcli.key().generate_with_seed_expect_fail(
        "ed25519-extended",
        seed,
        "invalid seed length, expected 32 bytes but received",
    );
}

#[test]
pub fn test_key_with_seed_with_unknown_symbol_generation() {
    let jcli: JCli = Default::default();
    let incorrect_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcay";
    jcli.key().generate_with_seed_expect_fail(
        "ed25519-extended",
        incorrect_seed,
        "invalid Hexadecimal",
    );
}
