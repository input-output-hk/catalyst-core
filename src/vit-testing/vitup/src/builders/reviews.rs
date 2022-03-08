use crate::Result;
use fake::faker::name::en::Name;
use fake::Fake;
use rand::rngs::OsRng;
use rand::RngCore;
use vit_servicing_station_lib::db::models::community_advisors_reviews::ReviewRanking;
use vit_servicing_station_tests::common::data::ProposalTemplate;
use vit_servicing_station_tests::common::data::ReviewTemplate;

pub struct ReviewGenerator {
    generator: OsRng,
    current_id: u32,
    assessors_per_proposal_count: u32,
}

impl ReviewGenerator {
    pub fn new(assessors_per_proposal_count: u32) -> ReviewGenerator {
        ReviewGenerator {
            generator: OsRng,
            current_id: 1,
            assessors_per_proposal_count,
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
                        reviews.push(self.generate_community_reviews(proposal, assessor));
                    } else {
                        reviews.push(self.generate_standard_reviews(proposal, assessor));
                    }
                    self.increment_id();
                } else {
                    return Err(crate::error::Error::NoChallengeIdFound {
                        proposal_id: proposal.proposal_id.clone(),
                    });
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
    ) -> ReviewTemplate {
        let assessor = assessor.into();
        self.generate_review(proposal, &assessor)
    }

    fn generate_standard_reviews<S: Into<String>>(
        &mut self,
        proposal: &ProposalTemplate,
        assessor: S,
    ) -> ReviewTemplate {
        let assessor = assessor.into();
        self.generate_review(proposal, &assessor)
    }

    fn generate_review<S: Into<String>>(
        &mut self,
        proposal: &ProposalTemplate,
        assessor: S,
    ) -> ReviewTemplate {
        ReviewTemplate {
            id: Some(self.current_id.to_string()),
            proposal_id: proposal.proposal_id.parse().unwrap(),
            impact_alignment_rating_given: (self.generator.next_u32() % 5) as i32,
            impact_alignment_note: fake::faker::lorem::en::Sentence(0..10).fake::<String>(),
            assessor: assessor.into(),
            auditability_note: fake::faker::lorem::en::Sentence(0..10).fake::<String>(),
            auditability_rating_given: (self.generator.next_u32() % 5) as i32,
            feasibility_note: fake::faker::lorem::en::Sentence(0..10).fake::<String>(),
            feasibility_rating_given: (self.generator.next_u32() % 5) as i32,
            ranking: match self.generator.next_u32() % 2 {
                0 => ReviewRanking::Excellent,
                _ => ReviewRanking::Good,
            },
        }
    }
}
