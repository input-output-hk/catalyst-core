use assert_fs::TempDir;
use std::path::{Path, PathBuf};

pub struct DeploymentTree {
    root: PathBuf,
}

impl DeploymentTree {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn root_path(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn database_path(&self) -> PathBuf {
        self.root.join("database.sqlite3")
    }

    pub fn genesis_path(&self) -> PathBuf {
        self.root.join("genesis.yaml")
    }

    pub fn block0_path(&self) -> PathBuf {
        self.root.join("block0.bin")
    }

    pub fn qr_codes_path(&self) -> PathBuf {
        self.root.join("qr-codes")
    }

    pub fn wallet_search_pattern(&self) -> String {
        format!("{}/wallet_*_*", self.root.display())
    }

    pub fn wallet_secret<S: Into<String>>(&self, alias: S) -> PathBuf {
        self.root.join(alias.into())
    }

    pub fn voting_token(&self) -> PathBuf {
        self.root.join("voting_token.txt")
    }
}

impl From<&TempDir> for DeploymentTree {
    fn from(temp_dir: &TempDir) -> Self {
        Self::new(temp_dir.path())
    }
}
