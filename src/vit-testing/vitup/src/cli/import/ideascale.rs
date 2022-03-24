use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;
use thiserror::Error;
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub enum ImportFromIdeascaleFormatCommand {
    Scores(ImportScores),
    Proposals(ImportProposals),
    Challenges(ImportChallenges),
    Reviews(ImportReviews),
}

impl ImportFromIdeascaleFormatCommand {
    pub fn exec(self) -> Result<(), Error> {
        std::env::set_var("RUST_BACKTRACE", "full");

        match self {
            Self::Scores(scores) => scores.exec()?,
            Self::Proposals(proposals) => proposals.exec()?,
            Self::Challenges(challenges) => challenges.exec()?,
            Self::Reviews(reviews) => reviews.exec()?,
        }

        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct ImportProposals {
    #[structopt(long = "input")]
    pub input: PathBuf,

    #[structopt(
        long = "output",
        default_value = "../resources/external/challenges.json"
    )]
    pub output: PathBuf,
}

impl ImportProposals {
    pub fn exec(self) -> Result<(), Error> {
        let data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(&self.input)?)?;
        let proposals = &data["proposals.csv"];
        let content = serde_json::to_string_pretty(&proposals)?;
        let mut file = File::create(self.output)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
#[derive(StructOpt, Debug)]
pub struct ImportChallenges {
    #[structopt(long = "input")]
    pub input: PathBuf,

    #[structopt(
        long = "output",
        default_value = "../resources/external/proposals.json"
    )]
    pub output: PathBuf,
}

impl ImportChallenges {
    pub fn exec(self) -> Result<(), Error> {
        let data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(&self.input)?)?;
        let challenges = &data["challenges.csv"];
        let content = serde_json::to_string_pretty(&challenges)?;
        let mut file = File::create(self.output)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct ImportScores {
    #[structopt(long = "input")]
    pub input: PathBuf,

    #[structopt(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,
}

impl ImportScores {
    pub fn exec(self) -> Result<(), Error> {
        let scores: Vec<InputScores> =
            serde_json::from_str(&jortestkit::file::read_file(&self.input)?)?;

        let mut proposals_data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(&self.proposals)?)?;

        for score in scores {
            let proposal = proposals_data
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|x| x["proposal_id"] == score.proposal_id)
                .ok_or_else(|| Error::CannotFindProposalWithId(score.proposal_id.to_string()))?;

            let rating_given: f32 = score
                .rating_given
                .parse()
                .map_err(|_| Error::CannotFindProposalWithId(score.proposal_id.to_string()))?;

            proposal["proposal_impact_score"] = ((rating_given * 100.0) as u32).to_string().into();
        }

        let content = serde_json::to_string_pretty(&proposals_data)?;
        let mut file = File::create(self.proposals)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot find proposal with id: {0}")]
    CannotFindProposalWithId(String),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    CannotParseRatingToFloat(#[from] std::num::ParseFloatError),
}

#[derive(Serialize, Deserialize)]
pub struct InputScores {
    pub proposal_id: String,
    pub rating_given: String,
}

#[derive(StructOpt, Debug)]
pub struct ImportReviews {
    #[structopt(long = "input")]
    pub input: PathBuf,

    #[structopt(long = "output")]
    pub output: PathBuf,
}

impl ImportReviews {
    pub fn exec(self) -> Result<(), Error> {
        let mut reviews_data: serde_json::Value =
            serde_json::from_str(&jortestkit::file::read_file(&self.input)?)?;

        for review in reviews_data.as_array_mut().unwrap() {
            review["impact_alignment_rating_given"] = review["impact_alignment_rating_given"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                .into();
            review["feasibility_rating_given"] = review["feasibility_rating_given"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                .into();
            review["auditability_rating_given"] = review["auditability_rating_given"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                .into();
        }

        let content = serde_json::to_string_pretty(&reviews_data)?;
        let mut file = File::create(self.output)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
