mod command;
pub mod fake;

pub use command::{CardanoCliCommand, Error as CardanoCliCommandError};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CardanoCliMock {
    path: PathBuf,
}

impl Default for CardanoCliMock {
    fn default() -> Self {
        Self {
            path: Path::new("cardano-cli-mock").to_path_buf(),
        }
    }
}

impl CardanoCliMock {
    pub fn path(&self) -> PathBuf {
        self.path.to_path_buf()
    }
}
