use path_slash::PathBufExt as _;
use path_slash::PathExt as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub const QR_CODES: &str = "qr-codes";
pub const WALLET: &str = "wallet";
pub const COMMITTEE: &str = "committee";
pub const COMMITTEES: &str = "committees";

pub const PRIVATE_KEYS: &str = "private_keys";
pub const PRIVATE_DATA: &str = "private_data";
pub const BLOCKCHAIN: &str = "blockchain";
pub const NETWORK: &str = "network";

#[derive(Serialize, Deserialize)]
pub struct FolderDump {
    content: HashMap<String, Vec<String>>,
    root: PathBuf,
    blockchain_items: Vec<String>,
}

impl FolderDump {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let mut content = HashMap::new();

        content.insert(QR_CODES.to_owned(), Vec::new());
        content.insert(PRIVATE_KEYS.to_owned(), Vec::new());
        content.insert(PRIVATE_DATA.to_owned(), Vec::new());
        content.insert(BLOCKCHAIN.to_owned(), Vec::new());
        content.insert(NETWORK.to_owned(), Vec::new());

        Self {
            content,
            root: root.as_ref().to_path_buf(),
            blockchain_items: vec!["block0.bin".to_owned(), "genesis.yaml".to_owned()],
        }
    }

    pub fn push_qr_code<S: Into<String>>(&mut self, value: S) {
        self.content.get_mut(QR_CODES).unwrap().push(value.into());
    }
    pub fn push_wallet_private_keys<S: Into<String>>(&mut self, value: S) {
        self.content
            .get_mut(PRIVATE_KEYS)
            .unwrap()
            .push(value.into());
    }
    pub fn push_blockchain<S: Into<String>>(&mut self, value: S) {
        self.content.get_mut(BLOCKCHAIN).unwrap().push(value.into());
    }
    pub fn push_private_data_keys<S: Into<String>>(&mut self, value: S) {
        self.content
            .get_mut(PRIVATE_DATA)
            .unwrap()
            .push(value.into());
    }
    pub fn push_network<S: Into<String>>(&mut self, value: S) {
        self.content.get_mut(NETWORK).unwrap().push(value.into());
    }

    pub fn push_path<P: AsRef<Path>>(&mut self, input: P) {
        let path = input.as_ref().to_slash().unwrap();
        let replacer = self.root.to_slash().unwrap();
        let path = path.replace(&format!("{}/", replacer), "");

        if path.contains(QR_CODES) {
            self.push_qr_code(path);
        } else if path.starts_with(WALLET) || path == COMMITTEE {
            self.push_wallet_private_keys(path);
        } else if self.blockchain_items.contains(&path) {
            self.push_blockchain(path);
        } else if path.contains(COMMITTEES) {
            self.push_private_data_keys(path);
        } else {
            self.push_network(path);
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("root folder does not exist yet. try to start backend")]
    RootFolderDoesNotExist(#[from] walkdir::Error),
}

fn is_storage(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.contains("storage.db"))
        .unwrap_or(false)
}

pub fn dump_json<P: AsRef<Path>>(root: P) -> Result<FolderDump, Error> {
    let walker = WalkDir::new(root.as_ref()).into_iter();
    let mut data: FolderDump = FolderDump::new(root);

    for entry in walker.filter_entry(|e| !is_storage(e)) {
        let entry = entry?;
        let md = std::fs::metadata(entry.path()).unwrap();
        if md.is_dir() {
            continue;
        }

        data.push_path(&*entry.path().to_slash().unwrap());
    }

    Ok(data)
}
