use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

const UP: &str = "up.sql";
const DOWN: &str = "down.sql";
const VERSION: &str = "2020-05-22-112032_setup_db";

pub struct MigrationFilesBuilder {
    up_script_content: String,
    down_script_content: String,
}

impl Default for MigrationFilesBuilder {
    fn default() -> Self {
        Self {
            up_script_content: include_str!("../../resources/vit_station/up.sql").to_string(),
            down_script_content: include_str!("../../resources/vit_station/down.sql").to_string(),
        }
    }
}

impl MigrationFilesBuilder {
    pub fn build<P: AsRef<Path>>(self, working_dir: P) -> Result<PathBuf, Error> {
        let migrations_dir = working_dir.as_ref().to_path_buf().join("migrations");
        let version_dir = migrations_dir.join(VERSION);
        std::fs::create_dir_all(&version_dir)?;

        let up_path = version_dir.join(UP);
        let mut up_file =
            File::create(&up_path).map_err(|_| Error::CannotCreateFile(up_path.clone()))?;
        up_file
            .write_all(self.up_script_content.as_bytes())
            .map_err(|_| Error::CannotWriteToFile(up_path.clone()))?;

        let down_path = version_dir.join(DOWN);
        let mut server_file =
            File::create(&down_path).map_err(|_| Error::CannotWriteToFile(down_path.clone()))?;
        server_file.write_all(self.down_script_content.as_bytes())?;

        Ok(migrations_dir)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot create file: {0}")]
    CannotCreateFile(PathBuf),
    #[error("cannot write file: {0}")]
    CannotWriteToFile(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
