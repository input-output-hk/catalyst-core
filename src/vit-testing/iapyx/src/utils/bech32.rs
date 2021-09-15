use bech32;
use std::path::Path;

pub fn read_bech32(path: impl AsRef<Path>) -> Result<(String, Vec<bech32::u5>), bech32::Error> {
    let line = jortestkit::file::read_file(path);
    let line_without_special_characters = line.replace(&['\n', '\r'][..], "");
    bech32::decode(&line_without_special_characters)
}
