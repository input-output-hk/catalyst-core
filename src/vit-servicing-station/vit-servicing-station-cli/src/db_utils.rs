use std::fs;
use std::io;
use std::io::{Read, Write};

pub fn db_file_exists(db_url: &str) -> io::Result<()> {
    // check if db file exists
    if !std::path::Path::new(db_url).exists() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} url does not exists", db_url),
        ));
    }
    Ok(())
}

pub fn backup_db_file(db_url: &str) -> io::Result<tempfile::NamedTempFile> {
    db_file_exists(db_url)?;
    let mut tmp_file = tempfile::NamedTempFile::new()?;
    let content = fs::read(db_url)?;
    tmp_file.write_all(&content)?;
    Ok(tmp_file)
}

pub fn restore_db_file(backup_file: tempfile::NamedTempFile, db_url: &str) -> io::Result<()> {
    let mut backup_file = backup_file.reopen()?;
    let mut buff = Vec::new();
    backup_file.read_to_end(&mut buff)?;
    fs::write(db_url, &buff)
}

#[cfg(test)]
mod test {
    use crate::db_utils::{backup_db_file, restore_db_file};
    use std::{fs, io};

    #[test]
    fn backup_file() -> io::Result<()> {
        let file_path = "./tmp_db.db";
        let content = b"foo bar";
        let content_vec = content.to_vec();
        // create a file with some content
        fs::write(file_path, content)?;

        // backup the file
        let tmp_file = backup_db_file(file_path)?;

        // write nonsense in old file
        fs::write(file_path, b"bar foo")?;

        // restore file and read content, hopefully is the old one
        restore_db_file(tmp_file, file_path)?;
        let backup_content = fs::read(file_path)?;
        fs::remove_file(file_path)?;

        // check written and actual content
        assert_eq!(&content_vec, &backup_content);

        Ok(())
    }
}
