mod command;
/// Arbitrary responses from voter registartion mock
pub mod fake;

use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use cardano_serialization_lib::GeneralTransactionMetadata;
use command::PATH_TO_DYNAMIC_CONTENT;
pub use command::{Command, Error};
use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Voter registration mock
#[derive(Debug, Clone)]
pub struct Mock {
    path: PathBuf,
}

impl Default for Mock {
    fn default() -> Self {
        Self {
            path: Path::new("voter-registration-mock").to_path_buf(),
        }
    }
}

impl Mock {
    /// Set response which will be returned when executing registration generation
    ///
    /// # Panics
    ///
    /// On IO related errors
    #[must_use]
    pub fn with_response(self, metadata: &GeneralTransactionMetadata, temp_dir: &TempDir) -> Self {
        let metadata_file = temp_dir.child("metadata.tmp");

        env::set_var(
            PATH_TO_DYNAMIC_CONTENT,
            metadata_file.path().to_str().unwrap(),
        );

        println!(
            "VoterRegistrationMock set dynamic response: {:?}",
            metadata_file.path().display()
        );

        let mut file = std::fs::File::create(metadata_file.path()).unwrap();
        file.write_all(serde_json::to_string(&metadata).unwrap().as_bytes())
            .unwrap();
        self
    }

    /// Path to persisted response
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
