use chain_impl_mockchain::block::BlockDateParseError;
use csv;
use jormungandr_lib::interfaces::BlockDate;
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct ArchiverRecord {
    pub fragment_id: String,
    pub caster: String,
    pub proposal: u32,
    pub time: String,
    pub choice: u8,
    pub raw_fragment: String,
}

impl ArchiverRecord {
    pub fn block_date(&self) -> Result<BlockDate, BlockDateParseError> {
        self.time.parse()
    }
}

pub fn load_from_csv<P: AsRef<Path>>(
    csv_path: P,
) -> Result<Vec<ArchiverRecord>, ArchiveReaderError> {
    let csv_path = csv_path.as_ref();
    println!("reading {:?}", csv_path.to_path_buf());

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(true)
        .quoting(true)
        .quote(b'"')
        .trim(csv::Trim::All)
        .from_path(csv_path)?;

    let mut results = Vec::new();
    for record in reader.deserialize() {
        match record {
            Ok(data) => {
                results.push(data);
            }
            Err(e) => return Err(ArchiveReaderError::Csv(e)),
        }
    }
    Ok(results)
}

pub fn load_from_folder<P: AsRef<Path>>(
    folder_path: P,
) -> Result<Vec<ArchiverRecord>, ArchiveReaderError> {
    let mut records = Vec::new();

    for entry in std::fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(extension) = path.extension().and_then(OsStr::to_str) {
            if extension == "csv" {
                records.extend(load_from_csv(path)?);
            }
        }
    }
    Ok(records)
}

#[derive(Debug, Error)]
pub enum ArchiveReaderError {
    #[error("general error")]
    General(#[from] std::io::Error),
    #[error("csv error")]
    Csv(#[from] csv::Error),
}
