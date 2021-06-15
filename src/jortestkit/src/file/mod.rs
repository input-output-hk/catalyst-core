#![allow(dead_code)]

use fs_extra::dir::{copy, CopyOptions};
use std::io::{prelude::*, BufReader};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

pub fn find_file<P: AsRef<Path>>(root: P, part_of_name: &str) -> Option<PathBuf> {
    for entry in fs::read_dir(root).expect("cannot read root directory") {
        let entry = entry.unwrap();
        if entry.file_name().to_str().unwrap().contains(part_of_name) {
            return Some(entry.path());
        }
    }
    None
}

pub fn read_file(path: impl AsRef<Path>) -> String {
    let contents = fs::read_to_string(path).expect("cannot read file");
    trim_new_line_at_end(contents)
}

fn trim_new_line_at_end(mut content: String) -> String {
    if content.ends_with('\n') {
        let len = content.len();
        content.truncate(len - 1);
    }
    content
}

pub fn make_readonly(path: &Path) {
    if !path.exists() {
        std::fs::File::create(&path).unwrap();
    }
    let mut perms = fs::metadata(path.as_os_str()).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(path.as_os_str(), perms).expect("cannot set permissions");
}

pub fn copy_folder(from: &Path, to: &Path, overwrite: bool) {
    let mut options = CopyOptions::new();
    options.overwrite = overwrite;
    copy(from, to, &options).expect("cannot copy folder");
}

pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q, preserve_old: bool) {
    if preserve_old {
        let mut old_to = to.as_ref().to_path_buf();
        old_to.set_file_name(format!(
            "old_{}",
            old_to.file_name().unwrap().to_str().unwrap()
        ));
        copy_file(to.as_ref(), old_to.as_path(), false);
    }
    fs::copy(from, to).expect("cannot copy files");
}

pub fn read_file_as_vector(
    path: impl AsRef<Path>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut output = Vec::new();
    for line in reader.lines() {
        output.push(line?);
    }
    Ok(output)
}

pub fn have_the_same_content<P: AsRef<Path>>(left: P, right: P) -> bool {
    read_file(left) == read_file(right)
}

pub fn get_file_as_byte_vec<P: AsRef<Path>>(filename: P) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = std::fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");

    buffer
}

pub fn append<P: AsRef<Path>, S: Into<String>>(filename: P, line: S) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename.as_ref())?;

    writeln!(file, "{}", line.into())
}

pub fn write_lines<P: AsRef<Path>, S: Into<String>>(
    filename: P,
    lines: Vec<S>,
) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename.as_ref())?;

    for line in lines {
        writeln!(file, "{}", line.into())?;
    }
    Ok(())
}
