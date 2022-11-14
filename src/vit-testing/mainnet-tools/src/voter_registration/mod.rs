mod command;
pub mod fake;

use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use command::PATH_TO_DYNAMIC_CONTENT;
pub use command::{Error, VoterRegistrationCommand};
use snapshot_lib::registration::VotingRegistration;
use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct VoterRegistrationMock {
    path: PathBuf,
}

impl Default for VoterRegistrationMock {
    fn default() -> Self {
        Self {
            path: Path::new("voter-registration-mock").to_path_buf(),
        }
    }
}

impl VoterRegistrationMock {
    pub fn with_response(
        self,
        voting_registration: VotingRegistration,
        temp_dir: &TempDir,
    ) -> Self {
        let metadata_file = temp_dir.child("metadata.tmp");

        env::set_var(
            PATH_TO_DYNAMIC_CONTENT,
            metadata_file.path().to_str().unwrap(),
        );

        println!(
            "VoterRegistrationMock set dynamic response: {:?}",
            metadata_file.path()
        );

        let mut file = std::fs::File::create(metadata_file.path()).unwrap();
        file.write_all(
            serde_json::to_string(&voting_registration)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        self
    }

    pub fn path(&self) -> PathBuf {
        self.path.to_path_buf()
    }
}
