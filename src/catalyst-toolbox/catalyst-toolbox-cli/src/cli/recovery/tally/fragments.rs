use std::fs;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use bincode::Options;

use crate::cli::recovery::tally::Error;
use jormungandr_lib::interfaces::PersistentFragmentLog;

struct FileFragments {
    reader: BufReader<fs::File>,
    file_path: PathBuf,
}

struct FileFragmentsIterator {
    reader: BufReader<fs::File>,
    file_path: PathBuf,
    counter: usize,
}

impl FileFragments {
    pub fn from_path(file_path: PathBuf) -> std::io::Result<Self> {
        fs::File::open(file_path.clone()).map(|file| Self {
            reader: BufReader::new(file),
            file_path,
        })
    }
}

impl IntoIterator for FileFragments {
    type Item = Result<PersistentFragmentLog, Error>;
    type IntoIter = FileFragmentsIterator;

    fn into_iter(self) -> Self::IntoIter {
        FileFragmentsIterator {
            reader: self.reader,
            file_path: self.file_path,
            counter: 0,
        }
    }
}

impl Iterator for FileFragmentsIterator {
    type Item = Result<PersistentFragmentLog, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // EOF reached when buffer is empty after reading successfully at least one.
        // Then we stop the iterator.
        if self.reader.buffer().is_empty() && self.counter != 0 {
            return None;
        }
        let codec = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes();
        let current = self.counter;
        self.counter += 1;
        Some(
            codec
                .deserialize_from(&mut self.reader)
                .map_err(|_| Error::DeserializeError {
                    file: self.file_path.to_string_lossy().to_string(),
                    entry: current,
                }),
        )
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
) -> io::Result<impl Iterator<Item = Result<PersistentFragmentLog, Error>>> {
    let mut handles = Vec::new();
    for entry in entries {
        handles.push(FileFragments::from_path(entry)?);
    }
    Ok(handles.into_iter().flat_map(|handle| handle.into_iter()))
}

pub fn load_fragments_from_folder_path(
    folder: &Path,
) -> io::Result<impl Iterator<Item = Result<PersistentFragmentLog, Error>>> {
    read_entries_from_files_path(get_fragments_log_files_path(folder)?)
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
