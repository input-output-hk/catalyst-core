use crate::common::registration::do_registration;
use assert_fs::TempDir;
//RC001
#[test]
pub fn full_registration_process() {
    let temp_dir = TempDir::new().unwrap();
    let result = do_registration(&temp_dir);

    result.assert_status_is_finished();
    result.assert_qr_equals_to_sk();
}
