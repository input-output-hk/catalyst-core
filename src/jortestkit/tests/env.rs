use jortestkit::prelude::{enhance_exe_name, find_exec};

#[test]
fn find_env() {
    assert!(find_exec(enhance_exe_name("cargo".as_ref())).is_some());
}
