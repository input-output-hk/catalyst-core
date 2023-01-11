use crate::builders::ReviewGenerator;
use crate::Result;
use std::io::Write;
use std::path::PathBuf;
use clap::Parser;
use vit_servicing_station_tests::common::data::ProposalTemplate;

#[derive(Parser, Debug)]
pub struct RandomReviewsDataCommandArgs {
    /// Careful! directory would be removed before export
    #[clap(long = "output", default_value = "./reviews.json")]
    pub output_file: PathBuf,

    #[clap(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    #[clap(long = "assessors-per-proposal-count", default_value = "3")]
    pub assessors_per_proposal_count: u32,
}

impl RandomReviewsDataCommandArgs {
    pub fn exec(self) -> Result<()> {
        let proposals: Vec<ProposalTemplate> =
            serde_json::from_str(&std::fs::read_to_string(&self.proposals)?)?;

        let mut generator = ReviewGenerator::new(self.assessors_per_proposal_count);
        let reviews = generator.generate(proposals)?;
        let content = serde_json::to_string_pretty(&reviews)?;
        let mut file = std::fs::File::create(&self.output_file)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
