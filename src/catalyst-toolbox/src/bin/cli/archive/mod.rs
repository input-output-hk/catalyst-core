use catalyst_toolbox::archive::generate_archive_files;

use structopt::StructOpt;

use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Archive {
    /// The path to the Jormungandr database to dump transactions from.
    jormungandr_database: PathBuf,
    /// CSV output directory
    output_dir: PathBuf,
}

impl Archive {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        generate_archive_files(&self.jormungandr_database, &self.output_dir).unwrap();
        Ok(())
    }
}
