use crate::common::registration::{do_registration, RegistrationResultAsserts};
use crate::common::snapshot::do_snapshot;
use crate::common::snapshot::wait_for_db_sync;
use crate::common::snapshot::RegistrationAsserts;
use assert_fs::TempDir;
use snapshot_trigger_service::config::JobParameters;
const GRACE_PERIOD_FOR_SNAPSHOT: u64 = 300;

//SR001
//SR003
#[test]
pub fn multiple_registration() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let first_registartion = do_registration(&temp_dir).as_legacy_registration().unwrap();
    first_registartion.status().assert_is_finished();
    first_registartion.assert_qr_equals_to_sk();

    let (overridden_identifier, _) = first_registartion.snapshot_entry().unwrap();

    println!("Waiting 10 mins before running next registration");
    std::thread::sleep(std::time::Duration::from_secs(5 * 60));
    println!("Wait finished.");

    let second_registration = do_registration(&temp_dir).as_legacy_registration().unwrap();
    second_registration.status().assert_is_finished();
    second_registration.assert_qr_equals_to_sk();

    let (identifier, value) = second_registration.snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(second_registration.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT),
        tag: None,
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    let snapshot_entries = snapshot_result.registrations();

    snapshot_entries.assert_contains_voting_key_and_value(&identifier, value);
    snapshot_entries.assert_not_contain_voting_key(&overridden_identifier);
}

///
///SR002
/// Test for catalyst-toolbox filter which should remove entry from snapshot with too low funds
#[test]
pub fn wallet_has_less_than_threshold() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let registration = do_registration(&temp_dir).as_legacy_registration().unwrap();
    registration.status().assert_is_finished();
    registration.assert_qr_equals_to_sk();

    let (too_low_funds_entry, _) = registration.snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(registration.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT),
        tag: None,
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    snapshot_result
        .registrations()
        .assert_not_contain_voting_key(&too_low_funds_entry);
}

//SR005
#[test]
pub fn wallet_with_funds_equals_to_threshold_should_be_elligible_to_vote() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let registration = do_registration(&temp_dir).as_legacy_registration().unwrap();
    registration.status().assert_is_finished();
    registration.assert_qr_equals_to_sk();

    let (id, _) = registration.snapshot_entry().unwrap();
    registration.print_snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(registration.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT),
        tag: None,
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    snapshot_result
        .registrations()
        .assert_not_contain_voting_key(&id);
}

//SR004
#[test]
pub fn registration_after_snapshot_is_not_taken_into_account() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let registration = do_registration(&temp_dir).as_legacy_registration().unwrap();
    registration.status().assert_is_finished();
    registration.assert_qr_equals_to_sk();

    let (too_late_id, _) = registration.snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(registration.slot_no().unwrap() - 1),
        tag: None,
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    let initials = snapshot_result.registrations();

    initials.assert_not_contain_voting_key(&too_late_id);
}
