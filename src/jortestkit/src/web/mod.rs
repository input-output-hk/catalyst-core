extern crate reqwest;
use std::fs::File;
use std::io;
use std::path::Path;
use thiserror::Error;

pub mod api_token;

#[derive(Debug, Error)]
pub enum WebError {
    #[error("could not download file")]
    CannotDownloadFile(#[from] reqwest::Error),
    #[error("could not save output to file")]
    CannotCreateOutputFile,
    #[error("could not send reqeuest")]
    IoError(#[from] io::Error),
}

pub fn download_file(link: String, output: &Path) -> Result<(), WebError> {
    let mut resp = reqwest::blocking::get(link).map_err(WebError::CannotDownloadFile)?;
    let mut out = File::create(output.as_os_str()).map_err(|_| WebError::CannotCreateOutputFile)?;
    io::copy(&mut resp, &mut out)?;
    Ok(())
}
