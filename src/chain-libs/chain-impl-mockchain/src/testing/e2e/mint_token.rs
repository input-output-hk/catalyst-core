use crate::{
    ledger::Error::MintingPolicyViolation,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, wallet},
        TestGen,
    },
    tokens::minting_policy::MintingPolicyViolation::AdditionalMintingNotAllowed,
};

const ALICE: &str = "ALICE";

#[test]
pub fn mint_token_not_allowed_outside_block_0() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new())
        .with_initials(vec![wallet(ALICE).with(1_000)])
        .build()
        .unwrap();

    let alice = controller.wallet(ALICE).unwrap();

    assert_eq!(
        controller
            .mint_token(
                &alice,
                TestGen::mint_token_for_wallet(alice.public_key().into()),
                &mut ledger,
            )
            .err()
            .unwrap(),
        MintingPolicyViolation(AdditionalMintingNotAllowed)
    );
}
