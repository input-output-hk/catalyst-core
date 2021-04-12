use crate::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use vit_servicing_station_tests::common::data::{parse_challenges, parse_funds, parse_proposals};
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct IdeascaleValidateCommand {
    /// proposals import json
    #[structopt(long = "output", default_value = "./validate/output")]
    pub output: PathBuf,

    /// proposals import json
    #[structopt(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges import json
    #[structopt(
        long = "challenges",
        default_value = "../resources/external/challenges.json"
    )]
    pub challenges: PathBuf,

    /// funds import json
    #[structopt(long = "funds", default_value = "../resources/external/funds.json")]
    pub funds: PathBuf,

    /// should i fix data is possible
    #[structopt(long = "fix")]
    pub fix: bool,
}

impl IdeascaleValidateCommand {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");
        std::fs::create_dir_all(&self.output)?;

        match parse_proposals(self.proposals.clone()) {
            Ok(_) => (),
            Err(err) => {
                if self.fix {
                    println!("Attempt to fix {:?}..", self.proposals);
                    self.try_to_fix_proposals()?;
                } else {
                    panic!("Error while parsing {:?}: {:?}", self.proposals, err);
                }
            }
        };
        parse_challenges(self.challenges)?;
        parse_funds(self.funds)?;

        Ok(())
    }

    pub fn try_to_fix_proposals(&self) -> Result<()> {
        let mut data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(&self.proposals))?;

        let proposals = data.as_array_mut().unwrap();
        for proposal in proposals.iter_mut() {
            let proposal_object = proposal.as_object_mut().unwrap();
            if !proposal_object.contains_key(&"challenge_type".to_string()) {
                if proposal_object.contains_key(&"proposal_brief".to_string()) {
                    proposal_object.insert(
                        "challenge_type".to_string(),
                        serde_json::Value::String("community-choice".to_string()),
                    );
                } else {
                    proposal_object.insert(
                        "challenge_type".to_string(),
                        serde_json::Value::String("simple".to_string()),
                    );
                }
            }
        }

        let content = serde_json::to_string_pretty(&data)?;
        let output =
            Path::new(&self.output).join(self.proposals.to_path_buf().file_name().unwrap());
        println!("Corrected proposals: {:?}..", output);
        let mut file = File::create(output)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
