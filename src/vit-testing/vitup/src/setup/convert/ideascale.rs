use crate::Result;
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;
use std::io::Write;
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct ConvertFromIdeascale {

    #[structopt(long = "input")]
    pub input: PathBuf,

    /// proposals output json
    #[structopt(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges output json
    #[structopt(
        long = "challenges",
        default_value = "../resources/external/challenges.json"
    )]
    pub challenges: PathBuf,

}

impl ConvertFromIdeascale {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let mut data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(&self.input))?;

        let proposals = &data["proposals.csv"];
        let challenges = &data["challenges.csv"];

        let content = serde_json::to_string_pretty(&proposals)?;
        let mut file = File::create(self.proposals)?;
        file.write_all(content.as_bytes())?;

        let content = serde_json::to_string_pretty(&challenges)?;
        let mut file = File::create(self.challenges)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
