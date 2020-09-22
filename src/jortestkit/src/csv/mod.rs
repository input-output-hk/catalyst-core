use csv::Writer;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

pub struct CsvFileBuilder {
    file: PathBuf,
    header: Vec<String>,
    content: Vec<Vec<String>>,
}

impl CsvFileBuilder {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            file: PathBuf::from(path.as_ref()),
            header: Vec::new(),
            content: Vec::new(),
        }
    }

    pub fn with_header(&mut self, header: Vec<&str>) -> &mut Self {
        self.header = header.iter().map(|x| x.to_string()).collect();
        self
    }

    pub fn with_content_line(&mut self, line: Vec<String>) -> &mut Self {
        self.content.push(line);
        self
    }

    pub fn with_contents(&mut self, content: Vec<Vec<String>>) -> &mut Self {
        for line in content {
            self.with_content_line(line);
        }
        self
    }

    pub fn build(&self) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(&self.file)?;
        wtr.write_record(&self.header)?;
        for line in self.content.iter() {
            wtr.write_record(&*line)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
