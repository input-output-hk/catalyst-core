use crate::common::registration::do_registration;
use crate::common::snapshot::do_snapshot;
use crate::common::snapshot::wait_for_db_sync;
use assert_fs::TempDir;
use jormungandr_automation::testing::asserts::InitialsAssert;
use snapshot_trigger_service::config::JobParameters;
const GRACE_PERIOD_FOR_SNAPSHOT: u64 = 300;
//SR001
//SR003
#[test]
pub fn multiple_registration() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let first_registartion = do_registration(&temp_dir);
    first_registartion.assert_status_is_finished();
    first_registartion.assert_qr_equals_to_sk();

    let overriden_entry = first_registartion.snapshot_entry().unwrap();

    println!("Waiting 10 mins before running next registration");
    std::thread::sleep(std::time::Duration::from_secs(5 * 60));
    println!("Wait finished.");

    let second_registartion = do_registration(&temp_dir);
    second_registartion.assert_status_is_finished();
    second_registartion.assert_qr_equals_to_sk();

    let correct_entry = second_registartion.snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(second_registartion.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT),
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    let initials = snapshot_result.initials();

    initials.assert_contains(correct_entry);
    initials.assert_not_contain(overriden_entry);
}

//SR002
#[test]
pub fn wallet_has_less_than_threshold() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let registartion = do_registration(&temp_dir);
    registartion.assert_status_is_finished();
    registartion.assert_qr_equals_to_sk();

    let too_low_funds_entry = registartion.snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(registartion.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT),
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    snapshot_result
        .initials()
        .assert_not_contain(too_low_funds_entry);
}

//SR005
#[test]
pub fn wallet_with_funds_equals_to_threshold_should_be_elligible_to_vote() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let registartion = do_registration(&temp_dir);
    registartion.assert_status_is_finished();
    registartion.assert_qr_equals_to_sk();

    let correct_entry = registartion.snapshot_entry().unwrap();
    registartion.print_snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(registartion.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT)
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    snapshot_result.initials().assert_contains(correct_entry);
}

//SR004
#[test]
pub fn registration_after_snapshot_is_not_taken_into_account() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let registartion = do_registration(&temp_dir);
    registartion.assert_status_is_finished();
    registartion.assert_qr_equals_to_sk();

    let too_late_entry = registartion.snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(registartion.slot_no().unwrap() - 1),
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    let initials = snapshot_result.initials();

    initials.assert_not_contain(too_late_entry);
}
