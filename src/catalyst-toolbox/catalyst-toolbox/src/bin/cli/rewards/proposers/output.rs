use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;

use super::types::Calculation;

pub fn build_path_for_challenge(path: &Path, challenge_name: &str) -> PathBuf {
    let ext = path.extension();
    let path = path.with_extension("");
    let path = path.join(format!("_{challenge_name}"));
    match ext {
        Some(ext) => path.with_extension(ext),
        None => path,
    }
}

pub fn write_json(path: &Path, results: &[Calculation]) -> Result<()> {
    let writer = BufWriter::new(File::options().write(true).open(path)?);
    serde_json::to_writer(writer, &results)?;

    Ok(())
}

pub fn write_csv(path: &Path, results: &[Calculation]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in results {
        writer.serialize(record)?;
    }
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_path() {
        let path = "/some/path.ext";
        let challenge = "challenge";
        let built_path = build_path_for_challenge(path.into(), challenge_name);
        assert_eq!(built_path, PathBuf::from("/some/path_challenge.ext"));
    }
}
