#![cfg(test)]

use crate::{
    chaintypes::ConsensusType,
    config::ConfigParam,
    date::BlockDate,
    fragment::{config::ConfigParams, Fragment},
    ledger::{
        ledger::{
            Block0Error,
            Error::{Block0, ExpectingInitialMessage},
        },
        Ledger,
    },
    milli::Milli,
    testing::{
        arbitrary::{AccountStatesVerifier, ArbitraryValidTransactionData, UtxoVerifier},
        builders::{OldAddressBuilder, TestTxBuilder},
        data::AddressDataValue,
        ledger::{ConfigBuilder, LedgerBuilder},
        TestGen,
    },
};

use chain_addr::Discrimination;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
pub fn ledger_accepts_correct_transaction(
    faucet: AddressDataValue,
    receiver: AddressDataValue,
) -> TestResult {
    let mut ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
        .initial_fund(&faucet)
        .build()
        .unwrap();
    let fragment = TestTxBuilder::new(ledger.block0_hash)
        .move_funds(&mut ledger, &faucet, &receiver, faucet.value)
        .get_fragment();
    let total_funds_before = ledger.total_funds();
    let result = ledger.apply_transaction(fragment, BlockDate::first());

    if result.is_err() {
        return TestResult::error(format!("Error from ledger: {}", result.err().unwrap()));
    }
    let total_funds_after = ledger.total_funds();
    if total_funds_before == total_funds_after {
        TestResult::passed()
    } else {
        TestResult::error(format!(
            "Total funds in ledger before and after transaction is not equal {} <> {} ",
            total_funds_before, total_funds_after
        ))
    }
}

#[quickcheck]
pub fn total_funds_are_const_in_ledger(
    transaction_data: ArbitraryValidTransactionData,
) -> TestResult {
    let config = ConfigBuilder::new(0)
        .with_discrimination(Discrimination::Test)
        .with_fee(transaction_data.fee);

    let mut ledger = LedgerBuilder::from_config(config)
        .initial_funds(&transaction_data.addresses)
        .build()
        .unwrap();
    let signed_tx = TestTxBuilder::new(ledger.block0_hash).move_funds_multiple(
        &mut ledger,
        &transaction_data.input_addresses,
        &transaction_data.output_addresses,
    );
    let total_funds_before = ledger.total_funds();
    let result = ledger.apply_transaction(signed_tx.get_fragment(), BlockDate::first());

    if result.is_err() {
        return TestResult::error(format!("Error from ledger: {:?}", result.err()));
    }

    let total_funds_after = ledger.total_funds();

    if total_funds_before != total_funds_after {
        return TestResult::error(format!(
            "Total funds in ledger before and after transaction is not equal {} <> {}",
            total_funds_before, total_funds_after
        ));
    }

    let utxo_verifier = UtxoVerifier::new(transaction_data.clone());
    let utxo_verification_result = utxo_verifier.verify(&ledger);
    if utxo_verification_result.is_err() {
        return TestResult::error(format!("{}", utxo_verification_result.err().unwrap()));
    }

    let account_state_verifier = AccountStatesVerifier::new(transaction_data);
    let account_state_verification_result = account_state_verifier.verify(ledger.accounts());
    if account_state_verification_result.is_err() {
        return TestResult::error(format!(
            "{}",
            account_state_verification_result.err().unwrap()
        ));
    }
    TestResult::passed()
}

#[test]
pub fn test_first_initial_fragment_empty() {
    let header_id = TestGen::hash();
    let content = Vec::new();
    assert_eq!(
        Ledger::new(header_id, content).err().unwrap(),
        Block0(Block0Error::InitialMessageMissing)
    );
}

#[test]
pub fn test_first_initial_fragment_wrong_type() {
    let header_id = TestGen::hash();
    let fragment = Fragment::OldUtxoDeclaration(OldAddressBuilder::build_utxo_declaration(Some(1)));
    assert_eq!(
        Ledger::new(header_id, &vec![fragment]).err().unwrap(),
        ExpectingInitialMessage
    );
}

#[test]
pub fn ledger_new_no_block_start_time() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));

    assert_eq!(
        Ledger::new(header_id, vec![&Fragment::Initial(ie)])
            .err()
            .unwrap(),
        Block0(Block0Error::InitialMessageNoDate)
    );
}

#[test]
pub fn ledger_new_dupicated_initial_fragments() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

    assert_eq!(
        Ledger::new(
            header_id,
            vec![&Fragment::Initial(ie.clone()), &Fragment::Initial(ie)]
        )
        .err()
        .unwrap(),
        Block0(Block0Error::InitialMessageMany)
    );
}

#[test]
pub fn ledger_new_duplicated_block0() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

    Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap();
}

#[test]
pub fn ledger_new_duplicated_discrimination() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

    Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap();
}

#[test]
pub fn ledger_new_duplicated_consensus_version() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::ConsensusVersion(ConsensusType::Bft));
    ie.push(ConfigParam::ConsensusVersion(ConsensusType::Bft));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

    Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap();
}

#[test]
pub fn ledger_new_duplicated_slot_duration() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotDuration(11u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

    Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap();
}

#[test]
pub fn ledger_new_duplicated_epoch_stability_depth() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::ConsensusVersion(ConsensusType::Bft));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::EpochStabilityDepth(10u32));
    ie.push(ConfigParam::EpochStabilityDepth(11u32));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

    Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap();
}

#[test]
pub fn ledger_new_duplicated_active_slots_coeff() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::ConsensusVersion(ConsensusType::Bft));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(
        Milli::from_millis(500),
    ));
    ie.push(ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(
        Milli::from_millis(600),
    ));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

    Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap();
}

#[test]
pub fn ledger_new_no_discrimination() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));

    assert_eq!(
        Ledger::new(header_id, vec![&Fragment::Initial(ie)])
            .err()
            .unwrap(),
        Block0(Block0Error::InitialMessageNoDiscrimination)
    );
}

#[test]
pub fn ledger_new_no_slot_duration() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));

    assert_eq!(
        Ledger::new(header_id, vec![&Fragment::Initial(ie)])
            .err()
            .unwrap(),
        Block0(Block0Error::InitialMessageNoSlotDuration)
    );
}

#[test]
pub fn ledger_new_no_slots_per_epoch() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::KesUpdateSpeed(3600));

    assert_eq!(
        Ledger::new(header_id, vec![&Fragment::Initial(ie)])
            .err()
            .unwrap(),
        Block0(Block0Error::InitialMessageNoSlotsPerEpoch)
    );
}

#[test]
pub fn ledger_new_no_kes_update_speed() {
    let leader_pair = TestGen::leader_pair();
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));

    assert_eq!(
        Ledger::new(header_id, vec![&Fragment::Initial(ie)])
            .err()
            .unwrap(),
        Block0(Block0Error::InitialMessageNoKesUpdateSpeed)
    );
}

#[test]
pub fn ledger_new_no_bft_leader() {
    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));

    assert_eq!(
        Ledger::new(header_id, vec![&Fragment::Initial(ie)])
            .err()
            .unwrap(),
        Block0(Block0Error::InitialMessageNoConsensusLeaderId)
    );
}

#[quickcheck]
pub fn wrong_fragment_at_block0(fragment: Fragment) -> TestResult {
    match fragment {
        Fragment::OldUtxoDeclaration(_) => return TestResult::discard(),
        Fragment::Transaction(_) => return TestResult::discard(),
        Fragment::StakeDelegation(_) => return TestResult::discard(),
        Fragment::PoolRegistration(_) => return TestResult::discard(),
        Fragment::VotePlan(_) => return TestResult::discard(),
        _ => (),
    };

    let header_id = TestGen::hash();
    let mut ie = ConfigParams::new();
    let leader_pair = TestGen::leader_pair();
    ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));
    ie.push(ConfigParam::Discrimination(Discrimination::Test));
    ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
    ie.push(ConfigParam::SlotDuration(10u8));
    ie.push(ConfigParam::SlotsPerEpoch(10u32));
    ie.push(ConfigParam::KesUpdateSpeed(3600));

    TestResult::from_bool(Ledger::new(header_id, vec![&Fragment::Initial(ie), &fragment]).is_err())
}
