use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use valgrind::Certs;

const SERVER_KEY: &str = "server.key";
const SERVER_CRT: &str = "server.crt";

pub struct CertificatesBuilder {
    server_crt_content: String,
    server_key_content: String,
}

impl Default for CertificatesBuilder {
    fn default() -> Self {
        Self {
            server_crt_content: include_str!("../../resources/tls/server.crt").to_string(),
            server_key_content: include_str!("../../resources/tls/server.key").to_string(),
        }
    }
}

impl CertificatesBuilder {
    pub fn build<P: AsRef<Path>>(self, working_dir: P) -> Result<Certs, Error> {
        let mut working_dir = working_dir.as_ref().to_path_buf();
        working_dir = working_dir.join("tls");
        std::fs::create_dir_all(&working_dir)?;

        let key_path = working_dir.join(SERVER_KEY);
        let mut key_file =
            File::create(&key_path).map_err(|_| Error::CannotCreateFile(key_path.clone()))?;
        key_file
            .write_all(self.server_key_content.as_bytes())
            .map_err(|_| Error::CannotWriteToFile(key_path.clone()))?;

        let cert_path = working_dir.join(SERVER_CRT);
        let mut server_file =
            File::create(&cert_path).map_err(|_| Error::CannotWriteToFile(cert_path.clone()))?;
        server_file.write_all(self.server_crt_content.as_bytes())?;

        Ok(Certs {
            key_path,
            cert_path,
        })
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
