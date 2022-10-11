use reqwest::Url;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ArchiveConfiguration {
    pub block_folder: PathBuf,
    pub archive_db: PathBuf,
}

pub fn discover_archive_input_files(url: impl Into<String>) -> Result<ArchiveConfiguration, Error> {
    let url = Url::parse(&url.into())?;
    match url.scheme() {
        "file" => get_configuration_from_file_url(
            url.to_file_path()
                .map_err(|()| DiscoverArchiveByFileError::IncorrectFolderUrl)?,
        )
        .map_err(Into::into),
        _ => unimplemented!("only file scheme is supported for archive location"),
    }
}

pub fn get_configuration_from_file_url(
    path: impl AsRef<Path>,
) -> Result<ArchiveConfiguration, DiscoverArchiveByFileError> {
    let path = path.as_ref();
    if !path.is_dir() {
        return Err(DiscoverArchiveByFileError::ExpectedUrlToFolder);
    }
    find_archive_db(path)
}

fn find_archive_db(
    path: impl AsRef<Path>,
) -> Result<ArchiveConfiguration, DiscoverArchiveByFileError> {
    let possible_db_extensions = vec!["db".to_string(), "database".to_string()];
    let possible_block0_extensions = vec!["bin".to_string()];

    let path = path.as_ref();

    let archive_db = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|entry| {
            if entry.path().is_dir() {
                return false;
            }

            let extension = entry.path().extension().and_then(OsStr::to_str).unwrap();
            possible_db_extensions.contains(&extension.to_string())
        })
        .map(|entry| entry.path().to_path_buf());

    let block_folder = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|entry| {
            if entry.path().is_dir() {
                return false;
            }

            let extension = entry.path().extension().and_then(OsStr::to_str).unwrap();
            possible_block0_extensions.contains(&extension.to_string())
        })
        .map(|entry| entry.path().parent().unwrap().to_path_buf());

    Ok(ArchiveConfiguration {
        archive_db: archive_db.ok_or(DiscoverArchiveByFileError::CannotFindArchiveDB {
            path: path.to_path_buf(),
            possible_extensions: possible_db_extensions,
        })?,
        block_folder: block_folder.ok_or(DiscoverArchiveByFileError::CannotFindBlock0Folder {
            path: path.to_path_buf(),
            possible_extensions: possible_block0_extensions,
        })?,
    })
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DiscoverArchiveByFile(#[from] DiscoverArchiveByFileError),
    #[error("parse url error")]
    Url(#[from] url::ParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum DiscoverArchiveByFileError {
    #[error(
        "cannot find archive db in path: {path}, expected extensions: {possible_extensions:?}"
    )]
    CannotFindArchiveDB {
        path: PathBuf,
        possible_extensions: Vec<String>,
    },
    #[error(
        "cannot find block0 folder in path: {path}, expected extensions: {possible_extensions:?}"
    )]
    CannotFindBlock0Folder {
        path: PathBuf,
        possible_extensions: Vec<String>,
    },
    #[error("incorrect file url ")]
    IncorrectFolderUrl,
    #[error("expected file url")]
    ExpectedUrlToFolder,
}
