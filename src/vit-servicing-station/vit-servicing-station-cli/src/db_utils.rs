use std::io;

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
