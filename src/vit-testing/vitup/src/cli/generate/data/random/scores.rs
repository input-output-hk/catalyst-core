use crate::Result;
use clap::Parser;
use rand::RngCore;
use rand_core::OsRng;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct RandomScoresDataCommandArgs {
    /// Careful! directory would be removed before export
    #[clap(long = "output", default_value = "./proposals.json")]
    pub output_file: PathBuf,

    #[clap(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    #[clap(long = "min", default_value = "100")]
    pub minimal_score: u32,

    #[clap(long = "max", default_value = "499")]
    pub maximal_score: u32,
}

impl RandomScoresDataCommandArgs {
    pub fn exec(self) -> Result<()> {
        let mut value: Vec<serde_json::Value> =
            serde_json::from_str(&std::fs::read_to_string(&self.proposals)?)?;

        let mut generator = OsRng;
        for proposal in value.iter_mut() {
            *proposal
                .as_object_mut()
                .unwrap()
                .get_mut("proposal_impact_score")
                .unwrap() = serde_json::Value::String(
                (generator.next_u32() % (self.maximal_score - self.minimal_score)
                    + self.minimal_score)
                    .to_string(),
            );
        }

        let content = serde_json::to_string_pretty(&value)?;
        let mut file = std::fs::File::create(&self.output_file)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
