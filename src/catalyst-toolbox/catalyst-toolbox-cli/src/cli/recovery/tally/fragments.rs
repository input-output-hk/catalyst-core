use std::fs;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use bincode::Options;
use jormungandr_lib::interfaces::PersistentFragmentLog;

struct FileFragments {
    reader: BufReader<fs::File>,
}

struct FileFragmentsIterator {
    reader: BufReader<fs::File>,
}

impl FileFragments {
    pub fn from_path(file_path: PathBuf) -> std::io::Result<Self> {
        fs::File::open(file_path).map(|file| Self {
            reader: BufReader::new(file),
        })
    }
}

impl IntoIterator for FileFragments {
    type Item = PersistentFragmentLog;
    type IntoIter = FileFragmentsIterator;

    fn into_iter(self) -> Self::IntoIter {
        FileFragmentsIterator {
            reader: self.reader,
        }
    }
}

impl Iterator for FileFragmentsIterator {
    type Item = PersistentFragmentLog;

    fn next(&mut self) -> Option<Self::Item> {
        let codec = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes();
        codec.deserialize_from(&mut self.reader).ok()
    }
}

pub fn get_fragments_log_files_path(folder: &Path) -> io::Result<impl Iterator<Item = PathBuf>> {
    let mut entries: Vec<_> = fs::read_dir(folder)?
        .filter_map(|entry| match entry {
            Ok(entry) => Some(folder.join(entry.path())),
            _ => None,
        })
        .collect();
    entries.sort();
    Ok(entries.into_iter())
}

pub fn read_entries_from_files_path(
    entries: impl Iterator<Item = PathBuf>,
) -> impl Iterator<Item = PersistentFragmentLog> {
    let handles = entries.into_iter().map(|path| {
        FileFragments::from_path(path.clone())
            .expect(&format!("Could not open: {}", path.to_string_lossy()))
    });
    handles.flat_map(|handle| handle.into_iter())
}

pub fn load_fragments_from_folder_path(
    folder: &Path,
) -> io::Result<impl Iterator<Item = PersistentFragmentLog>> {
    Ok(read_entries_from_files_path(get_fragments_log_files_path(
        folder,
    )?))
}

#[cfg(test)]
mod test {
    use crate::cli::recovery::tally::fragments::{
        get_fragments_log_files_path, load_fragments_from_folder_path,
    };
    use std::path::PathBuf;

    #[test]
    fn test_listing() -> std::io::Result<()> {
        let path: PathBuf = "D:/projects/rust/catalyst-toolbox/fragments_log"
            .parse()
            .unwrap();
        let fragments = load_fragments_from_folder_path(&path)?;
        for f in fragments {
            println!("{} -> {}", f.time, f.fragment.hash());
        }
        Ok(())
    }
}
