use color_eyre::Report;
use jormungandr_lib::interfaces::Initial;
use std::path::Path;

pub fn read_initials<S: Into<String>>(snapshot: S) -> Result<Vec<Initial>, Report> {
    let snapshot = snapshot.into();
    let value: serde_json::Value = serde_json::from_str(&snapshot)?;
    let initial = serde_json::to_string(&value["initial"])?;
    Ok(serde_json::from_str(&initial)?)
}

pub fn read_initials_from_file<P: AsRef<Path>>(initials: P) -> Result<Vec<Initial>, Report> {
    let contents = std::fs::read_to_string(&initials)?;
    read_initials(contents)
}
