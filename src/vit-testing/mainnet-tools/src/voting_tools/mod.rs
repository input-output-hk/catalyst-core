mod command;
pub mod fake;

use crate::db_sync::DbSyncInstance;
use crate::voting_tools::command::PATH_TO_DYNAMIC_CONTENT;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
pub use command::VotingToolsCommand;
use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct VotingToolsMock {
    path: PathBuf,
}

impl Default for VotingToolsMock {
    fn default() -> Self {
        Self {
            path: Path::new("voting-tools-mock").to_path_buf(),
        }
    }
}

impl VotingToolsMock {
    pub fn connect_to_db_sync(self, db_sync: &DbSyncInstance, temp_dir: &TempDir) -> Self {
        let voting_registrations = db_sync.query_all_registration_transactions();
        let snapshot_tmp = temp_dir.child("snapshot.tmp");

        env::set_var(
            PATH_TO_DYNAMIC_CONTENT,
            snapshot_tmp.path().to_str().unwrap(),
        );

        println!("{:?}", snapshot_tmp.path());

        let mut file = std::fs::File::create(snapshot_tmp.path()).unwrap();
        file.write_all(
            serde_json::to_string(&voting_registrations)
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
