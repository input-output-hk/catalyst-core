use crate::Result;
use crate::error::ErrorKind;
use fake::faker::name::en::Name;
use fake::Fake;
use rand::rngs::OsRng;
use rand::RngCore;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use vit_servicing_station_lib::db::models::community_advisors_reviews::ReviewTag;
use vit_servicing_station_tests::common::data::ProposalTemplate;
use vit_servicing_station_tests::common::data::ReviewTemplate;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct RandomReviewsDataCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "output", default_value = "./reviews.json")]
    pub output_file: PathBuf,

    #[structopt(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,
    
    #[structopt(
        long = "assessors-per-proposal-count",
        default_value = "3"
    )]
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

struct ReviewGenerator {
    generator: OsRng,
    current_id: u32,
    assessors_per_proposal_count: u32
}

impl ReviewGenerator {
    pub fn new(assessors_per_proposal_count: u32) -> ReviewGenerator {
        ReviewGenerator {
            generator: OsRng,
            current_id: 1,
            assessors_per_proposal_count
        }
    }

    pub fn generate(&mut self, proposals: Vec<ProposalTemplate>) -> Result<Vec<ReviewTemplate>> {
        let mut reviews = Vec::new();
        self.current_id = 1;
        for proposal in proposals.iter() {
            for _ in 0..self.assessors_per_proposal_count {
                let assessor = self.generate_assessor_name();
                if let Some(ref challenge_id) = &proposal.challenge_id {
                    if challenge_id.parse::<u32>().unwrap() == 1 {
                        reviews.extend(self.generate_community_reviews(proposal, assessor));
                    } else {
                        reviews.extend(self.generate_standard_reviews(proposal, assessor));
                    }
                } else {
                    bail!(ErrorKind::NoChallengeIdFound(proposal.proposal_id.clone()))
                }
            }
        }
        Ok(reviews)
    }

    fn increment_id(&mut self) {
        self.current_id += 1;
    }

    fn generate_assessor_name(&mut self) -> String {
        format!(
            "{}_{}",
            Name().fake::<String>().to_lowercase().replace(" ", "_"),
            (self.generator.next_u32() % 100)
        )
    }

    fn generate_community_reviews<S: Into<String>>(
        &mut self,
        proposal: &ProposalTemplate,
        assessor: S,
    ) -> Vec<ReviewTemplate> {
        let assessor = assessor.into();
        let mut reviews = vec![self.generate_review(proposal, &assessor, ReviewTag::Alignment)];
        self.increment_id();
        reviews.push(self.generate_review(proposal, &assessor, ReviewTag::Feasibility));
        self.increment_id();
        reviews.push(self.generate_review(proposal, &assessor, ReviewTag::Verifiability));
        self.increment_id();
        reviews
    }

    fn generate_standard_reviews<S: Into<String>>(
        &mut self,
        proposal: &ProposalTemplate,
        assessor: S,
    ) -> Vec<ReviewTemplate> {
        let assessor = assessor.into();
        let mut reviews = vec![self.generate_review(proposal, &assessor, ReviewTag::Alignment)];
        self.increment_id();
        reviews.push(self.generate_review(proposal, &assessor, ReviewTag::Auditability));
        self.increment_id();
        reviews.push(self.generate_review(proposal, &assessor, ReviewTag::Impact));
        self.increment_id();
        reviews
    }

    fn generate_review<S: Into<String>>(
        &mut self,
        proposal: &ProposalTemplate,
        assessor: S,
        tag: ReviewTag,
    ) -> ReviewTemplate {
        ReviewTemplate {
            id: Some(self.current_id.to_string()),
            proposal_id: proposal.proposal_id.parse().unwrap(),
            rating_given: (self.generator.next_u32() % 5) as i32,
            assessor: assessor.into(),
            note: fake::faker::lorem::en::Sentence(0..10).fake::<String>(),
            tag,
        }
    }
}
