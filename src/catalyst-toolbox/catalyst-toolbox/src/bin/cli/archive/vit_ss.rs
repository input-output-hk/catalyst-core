use catalyst_toolbox::archive::vit_ss::generate_archive_files;
use color_eyre::Report;
use std::path::{PathBuf, Path};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VitSS {
    /// The path to the vit servicing station database to dump data from.
    vit_ss_database: Path,
    /// CSV output directory
    output_dir: PathBuf,
}

impl VitSS {
    pub fn exec(self) -> Result<(), Report> {
        
        generate_archive_files(&self.vit_ss_database, &self.output_dir)?;
        Ok(())
    }
}
