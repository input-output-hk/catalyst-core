use chain_crypto::digest::DigestOf;
use chain_impl_mockchain::certificate::ExternalProposalId;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use typed_bytes::ByteBuilder;
use vit_servicing_station_tests::common::data::{
    parse_proposals, parse_reviews, ProposalTemplate, ReviewTemplate, TemplateLoad,
};

#[derive(Clone)]
pub struct ExternalProposalIdSource {
    pub proposals: PathBuf,
    pub reviews: PathBuf,
}

#[derive(Serialize, Clone)]
pub struct ProposalWithReviews {
    proposal: ProposalTemplate,
    reviews: Vec<ReviewTemplate>,
}

impl ProposalWithReviews {
    pub fn new(proposal: ProposalTemplate, mut reviews: Vec<ReviewTemplate>) -> Self {
        reviews.sort_by(|a, b| a.id.cmp(&b.id));

        Self { proposal, reviews }
    }
}

#[allow(clippy::from_over_into)]
impl Into<ExternalProposalId> for ProposalWithReviews {
    fn into(self) -> ExternalProposalId {
            let bb = ByteBuilder::new()
                .bytes(self.proposal.proposal_id.as_bytes())
                .bytes( self.proposal.proposal_url.as_bytes())
                .bytes( self.proposal.proposal_summary.as_bytes())
                .bytes( self.proposal.category_name.as_bytes())
                .bytes( self.proposal.challenge_type.to_string().as_bytes())
                .bytes( self.proposal.chain_vote_options.as_csv_string().as_bytes())
                .bytes( self.proposal.chain_vote_type.as_bytes())
                .bytes( self.proposal.files_url.as_bytes())
                .bytes( self.proposal.proposer_url.as_bytes())
                .finalize();
        DigestOf::digest_byteslice(&bb.as_byteslice())
    }
}

pub struct ProposalsWithReviewsCollection(Vec<ProposalWithReviews>);

impl ProposalsWithReviewsCollection {
    pub fn from_files<P: AsRef<Path>>(proposals: P, reviews: P) -> Result<Self, TemplateLoad> {
        let mut proposals: Vec<_> = parse_proposals(proposals.as_ref().to_path_buf())?.into_iter().collect();
        let mut reviews: Vec<_> = parse_reviews(reviews.as_ref().to_path_buf())?.into_iter().collect();

        proposals.sort_by(|a,b| a.proposal_id.cmp(&b.proposal_id));
        reviews.sort_by(|a,b| a.id.cmp(&b.id));

        Ok(Self(
            proposals
                .into_iter()
                .map(|p| {
                    let reviews = reviews
                        .iter()
                        .cloned()
                        .filter(|r| r.proposal_id == p.proposal_id)
                        .collect();
                    ProposalWithReviews::new(p, reviews)
                })
                .collect(),
        ))
    }
}

impl TryFrom<ExternalProposalIdSource> for ProposalsWithReviewsCollection {
    type Error = TemplateLoad;
    fn try_from(source: ExternalProposalIdSource) -> Result<Self, Self::Error> {
        Self::from_files(source.proposals, source.reviews)
    }
}

pub type ProposalsExternalIdMapping = Vec<(String, ExternalProposalId)>;

#[allow(clippy::from_over_into)]
impl From<ProposalsWithReviewsCollection> for ProposalsExternalIdMapping {
    fn from(collection: ProposalsWithReviewsCollection) -> Self {
        collection
            .0
            .into_iter()
            .map(|p| (p.proposal.proposal_id.clone(), p.into()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::builders::helpers::ProposalWithReviews;
    use chain_impl_mockchain::certificate::ExternalProposalId;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;
    use std::fmt::{Debug, Formatter};
    use std::iter;
    use vit_servicing_station_tests::common::data::{
        ArbitraryValidVotingTemplateGenerator, ValidVotingTemplateGenerator,
    };

    impl Debug for ProposalWithReviews {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str("")
        }
    }

    impl Arbitrary for ProposalWithReviews {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let len = usize::arbitrary(g) % 10 + 1;
            generate_proposal_with_review(len)
        }
    }

    fn generate_proposal_with_review(len: usize) -> ProposalWithReviews {
        let mut generator = ArbitraryValidVotingTemplateGenerator::default();
        let _ = generator.next_fund();
        let _ = generator.next_challenge();

        ProposalWithReviews {
            proposal: generator.next_proposal(),
            reviews: iter::from_fn(|| Some(generator.next_review()))
                .take(len)
                .collect(),
        }
    }

    #[quickcheck]
    pub fn external_id_is_repetitive_for_the_same_data(data: ProposalWithReviews) -> TestResult {
        let left: ExternalProposalId = data.clone().into();
        let right: ExternalProposalId = data.into();
        TestResult::from_bool(left == right)
    }

    #[test]
    pub fn external_id_is_different_for_different_reviews_order() {
        let mut proposal_with_review = generate_proposal_with_review(3);
        proposal_with_review.reviews.sort_by(|a, b| a.id.cmp(&b.id));

        let original: ExternalProposalId = proposal_with_review.clone().into();
        proposal_with_review.reviews.sort_by(|a, b| b.id.cmp(&a.id));
        let after: ExternalProposalId = proposal_with_review.into();

        assert!(original != after);
    }
}
