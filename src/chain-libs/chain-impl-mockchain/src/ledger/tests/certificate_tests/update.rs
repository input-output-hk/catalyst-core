use crate::{
    header::BlockDate,
    testing::arbitrary::update_proposal::UpdateProposalData,
    testing::{
        data::{AddressData, AddressDataValue, Wallet},
        scenario::FragmentFactory,
        ConfigBuilder, LedgerBuilder,
    },
    value::*,
};
use chain_addr::Discrimination;
use chain_core::property::Fragment;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
pub fn ledger_adopt_settings_from_update_proposal(
    update_proposal_data: UpdateProposalData,
) -> TestResult {
    let leader_pair = &update_proposal_data.leaders_pairs()[0];
    let mut leader = Wallet::from_address_data_value(AddressDataValue::new(
        AddressData::from_leader_pair(leader_pair.clone(), Discrimination::Test),
        Value(100),
    ));

    let cb = ConfigBuilder::new().with_leaders(&update_proposal_data.leaders_ids());
    let mut testledger = LedgerBuilder::from_config(cb)
        .faucets_wallets(vec![&leader])
        .build()
        .expect("cannot build test ledger");

    let fragment_factory = FragmentFactory::from_ledger(&testledger);
    let fragment = fragment_factory.update_proposal(
        testledger.date().next_epoch(),
        &leader,
        &leader,
        update_proposal_data.proposal.clone(),
    );

    leader.confirm_transaction();
    // apply proposal
    testledger
        .apply_fragment(&fragment, BlockDate::first().next_epoch())
        .unwrap();

    // apply votes
    for vote in update_proposal_data.gen_votes(fragment.id()) {
        let fragment =
            fragment_factory.update_vote(testledger.date().next_epoch(), &leader, &leader, vote);
        testledger
            .apply_fragment(&fragment, BlockDate::first().next_epoch())
            .unwrap();
        leader.confirm_transaction();
    }

    // trigger proposal process (build block)
    testledger
        .apply_empty_bft_block_with_date(leader_pair, testledger.date().next_epoch())
        .unwrap();

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
