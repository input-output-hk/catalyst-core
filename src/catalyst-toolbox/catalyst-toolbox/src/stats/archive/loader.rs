use color_eyre::eyre::bail;
use color_eyre::Report;
use csv;
use jormungandr_lib::interfaces::BlockDate;
use serde::Deserialize;
use std::{ffi::OsStr, fmt, path::Path};

#[derive(Debug, Deserialize)]
pub struct ArchiverRecord {
    pub fragment_id: String,
    pub caster: String,
    pub proposal: u32,
    #[serde(deserialize_with = "deserialize_block_date_from_float")]
    pub time: BlockDate,
    pub choice: u8,
    pub raw_fragment: String,
}

use serde::de::Visitor;
use serde::Deserializer;

pub fn deserialize_block_date_from_float<'de, D>(deserializer: D) -> Result<BlockDate, D::Error>
where
    D: Deserializer<'de>,
{
    struct VoteOptionsDeserializer();

    impl<'de> Visitor<'de> for VoteOptionsDeserializer {
        type Value = BlockDate;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("float in format {epoch}.{slod_id}")
        }

        fn visit_f64<E>(self, value: f64) -> Result<BlockDate, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_string().parse().unwrap())
        }
    }

    deserializer.deserialize_f64(VoteOptionsDeserializer())
}

pub fn load_from_csv<P: AsRef<Path>>(csv_path: P) -> Result<Vec<ArchiverRecord>, Report> {
    let csv_path = csv_path.as_ref();

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
            Err(e) => bail!("bad csv: {e}"),
        }
    }
    Ok(results)
}

pub fn load_from_folder<P: AsRef<Path>>(folder_path: P) -> Result<Vec<ArchiverRecord>, Report> {
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
