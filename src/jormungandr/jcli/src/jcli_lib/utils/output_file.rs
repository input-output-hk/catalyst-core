use crate::jcli_lib::utils::io;
use clap::Parser;
use std::{io::Write, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid output file path '{path}'")]
    CannotOpen {
        #[source]
        cause: std::io::Error,
        path: PathBuf,
    },
}

#[derive(Parser, Debug)]
pub struct OutputFile {
    /// output the key to the given file or to stdout if not provided
    #[clap(name = "OUTPUT_FILE")]
    output: Option<PathBuf>,
}

impl From<PathBuf> for OutputFile {
    fn from(output: PathBuf) -> Self {
        Self {
            output: Some(output),
        }
    }
}

impl OutputFile {
    pub fn open(&self) -> Result<impl Write, Error> {
        io::open_file_write(&self.output).map_err(|cause| Error::CannotOpen {
            cause,
            path: self.output.clone().unwrap_or_default(),
        })
    }

    /// Adds a prefix to the current path extension.
    /// For example, "my.long.filename.json" added ".is" becomes:
    /// "my.long.filename.is.json"
    #[must_use]
    pub fn extension_prefix(self, ext_prefix: &str) -> Self {
        match self.output {
            None => self,
            Some(ref path) => {
                let mut new_path: PathBuf = PathBuf::new();

                if let Some(path) = path.file_stem() {
                    new_path.push(path);
                };

                if let Some(ext) = path.extension() {
                    let ext =
                        ext_prefix.to_owned() + "." + ext.to_str().expect("Extension will exist.");

                    new_path.set_extension(ext);
                } else {
                    new_path.set_extension(ext_prefix);
                }

                Self {
                    output: Some(new_path),
                }
            }
        }
    }
}
