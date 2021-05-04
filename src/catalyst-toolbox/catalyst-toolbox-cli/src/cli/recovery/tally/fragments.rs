use std::fs;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use bincode::Options;
use jormungandr_lib::interfaces::PersistentFragmentLog;

pub struct FileFragments {
    reader: BufReader<fs::File>,
}

pub struct FileFragmentsIterator {
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
