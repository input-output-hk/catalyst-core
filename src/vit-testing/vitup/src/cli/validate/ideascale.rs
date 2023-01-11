use crate::builders::build_current_fund;
use crate::builders::utils::DeploymentTree;
use crate::config::Config;
use crate::mode::standard::DbGenerator;
use chain_impl_mockchain::testing::scenario::template::ProposalDefBuilder;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_impl_mockchain::testing::scenario::template::VotePlanDefBuilder;
use serde_json::Value;
use std::collections::LinkedList;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use clap::Parser;
use thiserror::Error;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vit_servicing_station_tests::common::data::{
    parse_challenges, parse_funds, parse_proposals, parse_reviews,
};
use vit_servicing_station_tests::common::data::{ChallengeTemplate, FundTemplate, ReviewTemplate};

#[derive(Parser, Debug)]
pub struct IdeascaleValidateCommand {
    /// proposals import json
    #[clap(long = "output", default_value = "./validate/output")]
    pub output: PathBuf,

    /// input folder
    #[clap(name = "INPUT")]
    pub input: PathBuf,

    /// prefix
    #[clap(long = "prefix")]
    pub prefix: Option<String>,

    /// should i fix data is possible
    #[clap(long = "fix")]
    pub fix: bool,

    /// should i fix data is possible
    #[clap(long = "mail", default_value = "")]
    pub mail_replacement: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("proposal error: ")]
    Proposal(#[from] ProposalError),
    #[error("challenge error: ")]
    Challenge(#[from] ChallengeError),
    #[error("review error: ")]
    Review(#[from] ReviewError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Template(#[from] vit_servicing_station_tests::common::data::TemplateLoad),
    #[error(transparent)]
    Data(#[from] crate::mode::standard::DataError),
}

#[derive(Debug, Error)]
pub enum ProposalError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error("illegal chars in proposal field: '{0}'")]
    IllegalChars(String),
    #[error("cannot find challenge with id: '{0}'")]
    CannotFindChallengeWithId(String),
    #[error("proposal funds illegal character: '{0}'")]
    ProposalFundsIllegalChar(String),
    #[error("empty proposal funds in proposal with id")]
    EmptyProposalFunds,
    #[error(transparent)]
    Template(#[from] vit_servicing_station_tests::common::data::TemplateLoad),
}

#[derive(Debug, Error)]
pub enum ReviewError {
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    Template(#[from] vit_servicing_station_tests::common::data::TemplateLoad),
}

#[derive(Debug, Error)]
pub enum ChallengeError {
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    Template(#[from] vit_servicing_station_tests::common::data::TemplateLoad),
    #[error("wrong fund in challenge id: {0}")]
    WrongFundId(String),
    #[error("zero rewards total in challenge: id: {0}")]
    ZeroRewardsTotal(String),
    #[error("rewards total mismatch with proposer rewards in challenge: {0}")]
    RewardsTotalMismatchWithProposersRewards(String),
    #[error("wrong challenge id: {0}")]
    CannotFindChallengeWithId(String),
}

impl IdeascaleValidateCommand {
    fn add_prefix(&self, file_name: &str) -> String {
        if let Some(prefix) = self.prefix.as_ref() {
            format!("{}{}", prefix, file_name)
        } else {
            file_name.to_string()
        }
    }

    pub fn exec(&self) -> Result<(), Error> {
        std::env::set_var("RUST_BACKTRACE", "full");
        std::fs::create_dir_all(&self.output)?;

        let proposals_path = self.input.join(self.add_prefix("proposals.json"));
        let funds_path = self.input.join(self.add_prefix("funds.json"));
        let challenges_path = self.input.join(self.add_prefix("challenges.json"));
        let reviews_path = self.input.join(self.add_prefix("reviews.json"));

        let mut funds = parse_funds(funds_path.clone())?;
        let challenges = self.validate_challenges(
            &challenges_path,
            &funds.pop_front().expect("empty funds collection"),
        )?;
        let mut proposals =
            self.validate_proposals(&proposals_path, self.fix, challenges.clone())?;
        let reviews = self.validate_reviews(&reviews_path)?;

        if self.fix {
            self.save_proposals(&proposals_path, &mut proposals)?;
            self.generate_vit_database(
                self.output.join(self.add_prefix("proposals.json")),
                challenges_path,
                funds_path,
                reviews_path,
                proposals.len(),
                challenges.len(),
                reviews.len(),
            )
        } else {
            self.generate_vit_database(
                proposals_path,
                challenges_path,
                funds_path,
                reviews_path,
                proposals.len(),
                challenges.len(),
                reviews.len(),
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_vit_database(
        &self,
        proposals_path: PathBuf,
        challenges_path: PathBuf,
        funds_path: PathBuf,
        reviews_path: PathBuf,
        proposals_count: usize,
        challenges_count: usize,
        reviews_count: usize,
    ) -> Result<(), Error> {
        std::fs::create_dir_all(&self.output)?;

        let deployment_tree = DeploymentTree::new(&self.output);

        let mut template_generator = ExternalValidVotingTemplateGenerator::new(
            proposals_path,
            challenges_path,
            funds_path,
            reviews_path,
        )?;

        let mut input_parameters = Config::default();

        input_parameters.data.current_fund.challenges = challenges_count;
        input_parameters.data.current_fund.reviews = reviews_count;
        input_parameters.data.current_fund.proposals = proposals_count as u32;

        let vote_plans = (0..proposals_count)
            .collect::<Vec<usize>>()
            .chunks(255)
            .map(|chunk| self.vote_plan_def(chunk.len()))
            .collect();

        let parameters = ValidVotePlanParameters::from(build_current_fund(
            &input_parameters,
            vote_plans,
            vec![],
        ));

        DbGenerator::new(parameters)
            .build(&deployment_tree.database_path(), &mut template_generator)?;

        Ok(())
    }

    fn vote_plan_def(&self, proposals_len: usize) -> VotePlanDef {
        let mut vote_plan_builder = VotePlanDefBuilder::new("fund_x");
        vote_plan_builder.owner("validate");
        vote_plan_builder.vote_phases(1, 2, 3);

        for _ in 0..proposals_len {
            let mut proposal_builder = ProposalDefBuilder::new(
                chain_impl_mockchain::testing::VoteTestGen::external_proposal_id(),
            );
            proposal_builder.options(3);
            proposal_builder.action_off_chain();
            vote_plan_builder.with_proposal(&mut proposal_builder);
        }

        vote_plan_builder.build()
    }

    fn validate_reviews<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<LinkedList<ReviewTemplate>, ReviewError> {
        parse_reviews(path.as_ref().to_path_buf()).map_err(ReviewError::Template)
    }

    fn validate_challenges<P: AsRef<Path>>(
        &self,
        path: P,
        fund: &FundTemplate,
    ) -> Result<LinkedList<ChallengeTemplate>, ChallengeError> {
        let challenges = parse_challenges(path.as_ref().to_path_buf())?;
        for challenge in challenges.iter() {
            if *challenge.fund_id.as_ref().unwrap() != fund.id.to_string() {
                return Err(ChallengeError::WrongFundId(challenge.id.to_string()));
            }
            if challenge.rewards_total.is_empty() {
                return Err(ChallengeError::ZeroRewardsTotal(challenge.id.to_string()));
            }
            if challenge.rewards_total != challenge.proposers_rewards {
                return Err(ChallengeError::RewardsTotalMismatchWithProposersRewards(
                    challenge.id.to_string(),
                ));
            }
        }
        Ok(challenges)
    }

    fn validate_proposals<P: AsRef<Path>>(
        &self,
        path: P,
        fix: bool,
        challenges: LinkedList<ChallengeTemplate>,
    ) -> Result<Vec<serde_json::Value>, ProposalError> {
        let path = path.as_ref();
        let mut proposals = match parse_proposals(path.to_path_buf()) {
            Ok(_) => self.parse_proposals_as_value(path)?,
            Err(err) => {
                if !self.fix {
                    return Err(ProposalError::Template(err));
                }
                println!("Attempt to fix {:?}..", path.to_path_buf());
                self.try_to_fix_proposals_missing_type(path)?
            }
        };

        self.check_and_eventually_fix_private_data(
            &mut proposals,
            fix,
            self.mail_replacement.clone(),
        );
        self.check_and_eventually_fix_proposal_funds(&mut proposals, fix)?;
        self.check_proposals_wrong_syntax(&mut proposals, challenges)?;
        Ok(proposals)
    }

    pub fn save_proposals<P: AsRef<Path>>(
        &self,
        path: P,
        data: &mut Vec<Value>,
    ) -> Result<(), ProposalError> {
        let content = serde_json::to_string_pretty(&data)?;
        let output = Path::new(&self.output).join(path.as_ref().to_path_buf().file_name().unwrap());
        println!("Corrected proposals: {:?}..", output);
        let mut file = File::create(output)?;
        file.write_all(content.as_bytes()).map_err(Into::into)
    }

    pub fn check_and_eventually_fix_private_data(
        &self,
        data: &mut [Value],
        fix: bool,
        mail_replacement: String,
    ) {
        for proposal in data.iter_mut() {
            if let Some(proposer_mail) = proposal.get_mut("proposer_email") {
                if fix {
                    let before = proposer_mail.as_str().unwrap();
                    let after = mail_replacement.clone();
                    println!("Sanitizing mail address {}-{}", before, after);
                    *proposer_mail = Value::String(after);
                } else {
                    panic!(
                        "not sanitized chars detected: {}",
                        proposer_mail.as_str().as_ref().unwrap()
                    );
                }
            }
        }
    }

    pub fn check_and_eventually_fix_proposal_funds(
        &self,
        data: &mut [Value],
        fix: bool,
    ) -> Result<(), ProposalError> {
        let illegal_char = ",";

        for proposal in data.iter_mut() {
            if let Some(proposal_funds) = proposal.get_mut("proposal_funds") {
                let before = proposal_funds.as_str().unwrap();

                if fix {
                    let after = before.replace(illegal_char, "");
                    println!(
                        "Fixing illegal chars in proposal funds {}-{}",
                        before, after
                    );
                    *proposal_funds = Value::String(after);
                } else {
                    if before.contains(illegal_char) {
                        return Err(ProposalError::ProposalFundsIllegalChar(
                            illegal_char.to_string(),
                        ));
                    }

                    if before.is_empty() {
                        return Err(ProposalError::EmptyProposalFunds);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn check_proposals_wrong_syntax(
        &self,
        data: &mut [Value],
        challenges: LinkedList<ChallengeTemplate>,
    ) -> Result<(), ProposalError> {
        for (idx, proposal) in data.iter_mut().enumerate() {
            let proposal_challenge_id = proposal
                .get_mut("challenge_id")
                .unwrap_or_else(|| panic!("cannot find challenge_id in proposal at {}", idx))
                .as_str()
                .unwrap();

            if !challenges.iter().any(|x| x.id == *proposal_challenge_id) {
                return Err(ProposalError::CannotFindChallengeWithId(
                    proposal_challenge_id.to_string(),
                ));
            }

            if let Some(proposal_solution) = proposal.get_mut("proposal_solution") {
                self.check_and_eventually_fix_proposal_syntax(proposal_solution)?;
            }

            if let Some(proposal_summary) = proposal.get_mut("proposal_summary") {
                self.check_and_eventually_fix_proposal_syntax(proposal_summary)?;
            }

            if let Some(proposer_relevant_experience) =
                proposal.get_mut("proposer_relevant_experience")
            {
                self.check_and_eventually_fix_proposal_syntax(proposer_relevant_experience)?;
            }

            if let Some(proposal_brief) = proposal.get_mut("proposal_brief") {
                self.check_and_eventually_fix_proposal_syntax(proposal_brief)?;
            }

            if let Some(proposal_goal) = proposal.get_mut("proposal_goal") {
                self.check_and_eventually_fix_proposal_syntax(proposal_goal)?;
            }

            if let Some(proposal_importance) = proposal.get_mut("proposal_importance") {
                self.check_and_eventually_fix_proposal_syntax(proposal_importance)?;
            }

            if let Some(proposal_metrics) = proposal.get_mut("proposal_metrics") {
                self.check_and_eventually_fix_proposal_syntax(proposal_metrics)?;
            }

            if let Some(proposal_summary) = proposal.get_mut("proposal_summary") {
                self.check_and_eventually_fix_proposal_syntax(proposal_summary)?;
            }
        }
        Ok(())
    }

    fn check_and_eventually_fix_proposal_syntax(
        &self,
        value: &mut Value,
    ) -> Result<(), ProposalError> {
        let illegal_chars_vec = vec!["**", "\\\\*", "\\\\", "\\*\\*", "\\-"];
        for illegal_chars in illegal_chars_vec {
            if value.as_str().as_ref().unwrap().contains(illegal_chars) {
                if self.fix {
                    let before = value.as_str().unwrap();
                    let after = value.as_str().as_mut().unwrap().replace(illegal_chars, "");
                    println!("Fixing illegal chars {}-{}", before, after);
                    *value = Value::String(after);
                } else {
                    return Err(ProposalError::IllegalChars(
                        value.as_str().as_ref().unwrap().to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn parse_proposals_as_value<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<Value>, ProposalError> {
        let mut data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(path)?)?;
        Ok(data.as_array_mut().unwrap().to_vec())
    }

    pub fn try_to_fix_proposals_missing_type<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<Value>, ProposalError> {
        let mut proposals = self.parse_proposals_as_value(path)?;
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
