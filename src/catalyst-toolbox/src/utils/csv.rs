use serde::de::DeserializeOwned;
use serde::Serialize;

use std::path::Path;

pub fn load_data_from_csv<T: DeserializeOwned>(file_path: &Path) -> Result<Vec<T>, csv::Error> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;
    csv_reader.deserialize().collect::<Result<Vec<T>, _>>()
}

pub fn dump_data_to_csv<T: Serialize>(data: &[T], file_path: &Path) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;
    for entry in data {
        writer.serialize(entry)?;
    }
    Ok(())
}
