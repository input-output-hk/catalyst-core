use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub fn load_data_from_csv<T: DeserializeOwned, const DELIMITER: u8>(
    file_path: &Path,
) -> Result<Vec<T>, csv::Error> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(DELIMITER)
        .from_path(file_path)?;
    csv_reader.deserialize().collect::<Result<Vec<T>, _>>()
}

pub fn dump_data_to_csv<'a, T: 'a + Serialize>(
    data: impl Iterator<Item = &'a T>,
    file_path: &Path,
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;
    for entry in data {
        writer.serialize(entry)?;
    }
    Ok(())
}

pub fn dump_to_csv_or_print<'a, T: 'a + Serialize + Debug>(
    output: Option<PathBuf>,
    result: impl Iterator<Item = &'a T> + Debug,
) -> Result<(), std::io::Error> {
    if let Some(output) = &output {
        dump_data_to_csv(result, output)?;
    } else {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(std::io::stdout());
        for entry in result {
            writer.serialize(entry)?;
        }
    }
    Ok(())
}
