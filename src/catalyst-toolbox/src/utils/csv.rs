use serde::de::DeserializeOwned;
use std::path::Path;

pub fn load_data_from_csv<T: DeserializeOwned>(file_path: &Path) -> Result<Vec<T>, csv::Error> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;
    let mut res = Vec::new();
    for entry in csv_reader.deserialize() {
        let entry: T = entry?;
        res.push(entry);
    }
    Ok(res)
}
