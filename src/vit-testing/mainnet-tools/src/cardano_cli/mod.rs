mod command;
/// Module for providing fake/arbitrary data which Cardano CLI can return.
/// Should be used purely for testing
pub mod fake;

pub use command::Command;
use std::path::{Path, PathBuf};

/// Cardano CLI mock. It can return arbitrary/fake data but preserves correct Cardano CLI format.
#[derive(Debug, Clone)]
pub struct Mock {
    path: PathBuf,
}

impl Default for Mock {
    fn default() -> Self {
        Self {
            path: Path::new("cardano-cli-mock").to_path_buf(),
        }
    }
}

impl Mock {
    /// Path to mock cli
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
