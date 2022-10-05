#![allow(dead_code)]

use fs_extra::dir::{copy, CopyOptions};
use std::io::{prelude::*, BufReader};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

pub fn find_file<P: AsRef<Path>>(
    root: P,
    part_of_name: &str,
) -> Result<Option<PathBuf>, std::io::Error> {
    for entry in fs::read_dir(root)? {
        let entry = entry.unwrap();
        if entry.file_name().to_str().unwrap().contains(part_of_name) {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

pub fn read_file(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    Ok(trim_new_line_at_end(contents))
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

pub fn copy_folder(from: &Path, to: &Path, overwrite: bool) -> Result<(), fs_extra::error::Error> {
    let mut options = CopyOptions::new();
    options.overwrite = overwrite;
    copy(from, to, &options).map(|_| ())
}

pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(
    from: P,
    to: Q,
    preserve_old: bool,
) -> Result<(), std::io::Error> {
    if preserve_old {
        let mut old_to = to.as_ref().to_path_buf();
        old_to.set_file_name(format!(
            "old_{}",
            old_to.file_name().unwrap().to_str().unwrap()
        ));
        return copy_file(to.as_ref(), old_to.as_path(), false);
    }
    fs::copy(from, to).map(|_| ())
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

pub fn have_the_same_content<P: AsRef<Path>>(left: P, right: P) -> Result<bool, std::io::Error> {
    Ok(read_file(left)? == read_file(right)?)
}

pub fn get_file_as_byte_vec<P: AsRef<Path>>(filename: P) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(&filename)?;
    let metadata = std::fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer)?;

    Ok(buffer)
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
