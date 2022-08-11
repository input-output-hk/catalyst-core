use crate::common::registration::{do_registration, RegistrationResultAsserts};
use assert_fs::TempDir;
//RC001
#[test]
pub fn full_registration_process() {
    let temp_dir = TempDir::new().unwrap();
    let result = do_registration(&temp_dir).as_legacy_registration().unwrap();

    result.status().assert_is_finished();
    result.assert_qr_equals_to_sk();
}
