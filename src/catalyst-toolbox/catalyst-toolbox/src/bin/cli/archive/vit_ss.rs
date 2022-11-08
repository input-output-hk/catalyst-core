use catalyst_toolbox::archive::vit_ss::generate_archive_files;
use color_eyre::Report;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::runtime::Runtime;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VitSS {
    /// The path to the vit servicing station database to dump data from.
    vit_ss_database: String,
    /// CSV output directory
    output_dir: PathBuf,
    // Fund id
    fund_id: i32,
}

impl VitSS {
    pub fn exec(self) -> Result<(), Report> {
        Runtime::new()?.block_on(async move {
            generate_archive_files(&self.vit_ss_database, &self.output_dir, self.fund_id).await
        })?;
        Ok(())
    }
}
