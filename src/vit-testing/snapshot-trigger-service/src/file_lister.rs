use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct FolderDump {
    content: Vec<String>,
    #[serde(skip_serializing)]
    root: PathBuf,
}

impl FolderDump {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            content: Vec::new(),
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn push<S: Into<String>>(&mut self, data: S) {
        let item = data.into();
        let root_file_name = format!("{}", self.root.display());
        self.content
            .push(item.replace(&root_file_name, "").replace("\\", "/"));
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("root folder does not exist yet. try to start backend")]
    RootFolderDoesNotExist(#[from] walkdir::Error),
}

pub fn dump_json<P: AsRef<Path>>(root: P) -> Result<FolderDump, Error> {
    let walker = WalkDir::new(root.as_ref()).into_iter();
    let mut data: FolderDump = FolderDump::new(root);

    for entry in walker {
        let entry = entry?;
        let md = std::fs::metadata(entry.path()).unwrap();
        if !md.is_dir() {
            data.push(format!("{}", entry.path().display()));
        }
    }
    Ok(data)
}
