use crate::{
    account::SpendingCounter,
    block::{self, Block},
    chaintypes::HeaderId,
    date::BlockDate,
    fragment::Contents,
    header::BlockVersion,
    key::EitherEd25519SecretKey,
    ledger::ledger::Ledger,
    testing::arbitrary::update_proposal::UpdateProposalData,
    testing::{
        builders::TestTxCertBuilder,
        data::{AddressData, AddressDataValue, Wallet},
        ConfigBuilder, LedgerBuilder,
    },
    value::*,
};
use chain_addr::{Address, Discrimination, Kind};
use chain_core::property::Fragment;
use chain_crypto::{Ed25519, SecretKey};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
pub fn ledger_adopt_settings_from_update_proposal(
    update_proposal_data: UpdateProposalData,
) -> TestResult {
    let leader_pair = update_proposal_data.leaders.iter().next().unwrap();
    let mut leader = Wallet::from_address_data_value(AddressDataValue::new(
        AddressData::new(
            EitherEd25519SecretKey::Normal(leader_pair.1.clone()),
            Some(SpendingCounter(0)),
            Address(
                Discrimination::Test,
                Kind::Account(leader_pair.0.as_public_key().clone()),
            ),
        ),
        Value(100),
    ));

    let cb = ConfigBuilder::new().with_leaders(&update_proposal_data.leaders_ids());
    let mut testledger = LedgerBuilder::from_config(cb)
        .faucets_wallets(vec![&leader])
        .build()
        .expect("cannot build test ledger");

    let fragment = TestTxCertBuilder::new(testledger.block0_hash, testledger.fee())
        .make_transaction(
            testledger.date().next_epoch(),
            &[leader.clone()],
            &update_proposal_data.proposal.clone().into(),
        );

    leader.confirm_transaction();
    // apply proposal
    testledger
        .apply_fragment(&fragment, BlockDate::first().next_epoch())
        .unwrap();

    // apply votes
    for vote in update_proposal_data.gen_votes(fragment.id()) {
        let fragment = TestTxCertBuilder::new(testledger.block0_hash, testledger.fee())
            .make_transaction(testledger.date().next_epoch(), vec![&leader], &vote.into());
        testledger
            .apply_fragment(&fragment, BlockDate::first().next_epoch())
            .unwrap();
        leader.confirm_transaction();
    }

    // trigger proposal process (build block)
    let block = build_block(
        &testledger.ledger,
        testledger.block0_hash,
        testledger.ledger.date().next_epoch(),
        &update_proposal_data.block_signing_key,
    );
    testledger.apply_block(block).unwrap();

    // assert
    let actual_params = testledger.ledger.settings.to_config_params();
    let expected_params = update_proposal_data.proposal_settings();

    let mut all_settings_equal = true;
    for expected_param in expected_params.iter() {
        if !actual_params.iter().any(|x| x == expected_param) {
            all_settings_equal = false;
            break;
        }
    }

    if !testledger.ledger.updates.proposals.is_empty() {
        return TestResult::error(format!(
            "Error: proposal collection should be empty but contains:{:?}",
            testledger.ledger.updates.proposals
        ));
    }

    if all_settings_equal {
        TestResult::passed()
    } else {
        TestResult::error(format!("Error: proposed update reached required votes, but proposal was NOT updated, Expected: {:?} vs Actual: {:?}",
                                expected_params,actual_params))
    }
}

fn build_block(
    ledger: &Ledger,
    block0_hash: HeaderId,
    date: BlockDate,
    block_signing_key: &SecretKey<Ed25519>,
) -> Block {
    let contents = Contents::empty();
    block::builder(BlockVersion::Ed25519Signed, contents, |header_builder| {
        Ok::<_, ()>(
            header_builder
                .set_parent(&block0_hash, ledger.chain_length.increase())
                .set_date(date.next_epoch())
                .into_bft_builder()
                .unwrap()
                .sign_using(block_signing_key)
                .generalize(),
        )
    })
    .unwrap()
}
