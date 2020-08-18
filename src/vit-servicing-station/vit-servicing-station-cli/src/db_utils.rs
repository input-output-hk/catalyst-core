use std::fs;
use std::io;
use std::io::{Read, Write};
use tempfile;

pub fn db_file_exists(db_url: &str) -> io::Result<()> {
    // check if db file exists
    if !std::path::Path::new(db_url).exists() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} url does not exists", db_url.to_string()),
        ));
    }
    Ok(())
}

pub fn backup_db_file(db_url: &str) -> io::Result<fs::File> {
    db_file_exists(db_url)?;
    let mut tmp_file = tempfile::tempfile()?;
    tmp_file.write_all(&fs::read(db_url)?)?;
    Ok(tmp_file)
}

pub fn restore_db_file(mut backup_file: fs::File, db_url: &str) -> io::Result<()> {
    let mut buff = Vec::new();
    backup_file.read(&mut buff)?;
    fs::write(db_url, &buff)
}
