use crate::http::HttpClient;
use color_eyre::eyre::Result;
use std::{fs::File, path::Path};

use serde::Deserialize;

pub fn json_from_file<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> Result<T> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

pub fn json_from_network<T: for<'a> Deserialize<'a>>(
    http: &impl HttpClient,
    url: impl AsRef<str>,
) -> Result<T> {
    http.get(url.as_ref())?.json()
}
