use crate::Result;
use serde_json::Value;
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

        let mut proposals = match parse_proposals(self.proposals.clone()) {
            Ok(_) => self.parse_proposals_as_value()?,
            Err(err) => {
                if !self.fix {
                    panic!("Error while parsing {:?}: {:?}", self.proposals, err);
                }
                println!("Attempt to fix {:?}..", self.proposals);
                self.try_to_fix_proposals_missing_type()?
            }
        };
        self.check_proposals_wrong_syntax(&mut proposals);
        self.check_and_eventually_fix_proposal_funds(&mut proposals);

        if self.fix {
            self.save_proposals(&mut proposals)?;
        }
        parse_challenges(self.challenges)?;
        parse_funds(self.funds)?;

        Ok(())
    }

    pub fn save_proposals(&self, data: &mut Vec<Value>) -> Result<()> {
        let content = serde_json::to_string_pretty(&data)?;
        let output =
            Path::new(&self.output).join(self.proposals.to_path_buf().file_name().unwrap());
        println!("Corrected proposals: {:?}..", output);
        let mut file = File::create(output)?;
        file.write_all(content.as_bytes()).map_err(Into::into)
    }

    pub fn check_and_eventually_fix_proposal_funds(&self, data: &mut Vec<Value>) {
        for proposal in data.iter_mut() {
            if let Some(proposal_funds) = proposal.get_mut("proposal_funds") {
                if self.fix {
                    let before = proposal_funds.as_str().unwrap();
                    let after = proposal_funds.as_str().as_mut().unwrap().replace(",", "");
                    println!(
                        "Fixing illegal chars in proposal funds {}-{}",
                        before, after
                    );
                    *proposal_funds = Value::String(after);
                }
            }
        }
    }

    pub fn check_proposals_wrong_syntax(&self, data: &mut Vec<Value>) {
        for proposal in data.iter_mut() {
            if let Some(proposal_solution) = proposal.get_mut("proposal_solution") {
                self.check_and_eventually_fix_proposal_syntax(proposal_solution);
            }

            if let Some(proposal_summary) = proposal.get_mut("proposal_summary") {
                self.check_and_eventually_fix_proposal_syntax(proposal_summary);
            }

            if let Some(proposer_relevant_experience) =
                proposal.get_mut("proposer_relevant_experience")
            {
                self.check_and_eventually_fix_proposal_syntax(proposer_relevant_experience);
            }

            if let Some(proposal_brief) = proposal.get_mut("proposal_brief") {
                self.check_and_eventually_fix_proposal_syntax(proposal_brief);
            }

            if let Some(proposal_goal) = proposal.get_mut("proposal_goal") {
                self.check_and_eventually_fix_proposal_syntax(proposal_goal);
            }

            if let Some(proposal_importance) = proposal.get_mut("proposal_importance") {
                self.check_and_eventually_fix_proposal_syntax(proposal_importance);
            }

            if let Some(proposal_metrics) = proposal.get_mut("proposal_metrics") {
                self.check_and_eventually_fix_proposal_syntax(proposal_metrics);
            }

            if let Some(proposal_summary) = proposal.get_mut("proposal_summary") {
                self.check_and_eventually_fix_proposal_syntax(proposal_summary);
            }
        }
    }

    fn check_and_eventually_fix_proposal_syntax(&self, value: &mut Value) {
        let illegal_chars_vec = vec!["**", "\\\\*", "\\\\", "\\*\\*", "\\-"];
        for illegal_chars in illegal_chars_vec {
            if value.as_str().as_ref().unwrap().contains(illegal_chars) {
                if self.fix {
                    let before = value.as_str().unwrap();
                    let after = value.as_str().as_mut().unwrap().replace(illegal_chars, "");
                    println!("Fixing illegal chars {}-{}", before, after);
                    *value = Value::String(after);
                } else {
                    panic!(
                        "illegal chars detected: {}",
                        value.as_str().as_ref().unwrap()
                    );
                }
            }
        }
    }

    fn parse_proposals_as_value(&self) -> Result<Vec<Value>> {
        let mut data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(&self.proposals))?;
        Ok(data.as_array_mut().unwrap().to_vec())
    }

    pub fn try_to_fix_proposals_missing_type(&self) -> Result<Vec<Value>> {
        let mut proposals = self.parse_proposals_as_value()?;
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
        Ok(proposals.to_vec())
    }
}
