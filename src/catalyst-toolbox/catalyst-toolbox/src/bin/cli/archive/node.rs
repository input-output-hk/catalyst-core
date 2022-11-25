use catalyst_toolbox::archive::node::generate_archive_files;
use color_eyre::Report;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Node {
    /// The path to the Jormungandr database to dump transactions from.
    jormungandr_database: PathBuf,
    /// CSV output directory
    output_dir: PathBuf,
}

impl Node {
    pub fn exec(self) -> Result<(), Report> {
        generate_archive_files(&self.jormungandr_database, &self.output_dir)?;
        Ok(())
    }
}
