use std::{
    fs::File,
    io::{prelude::*, BufReader},
    path::PathBuf,
};

pub struct Logger {
    log_file: PathBuf,
}

impl Logger {
    pub fn new(log_file: PathBuf) -> Self {
        Self { log_file }
    }

    pub fn log_file(&self) -> &PathBuf {
        &self.log_file
    }

    fn log_lines(&self) -> Vec<String> {
        let file = File::open(self.log_file()).expect("logger file not found");
        let buf = BufReader::new(file);
        buf.lines().map(|l| l.unwrap()).collect()
    }

    pub fn any_error(&self) -> bool {
        self.log_lines().iter().any(|x| x.contains(&"[ERROR]"))
    }
}
