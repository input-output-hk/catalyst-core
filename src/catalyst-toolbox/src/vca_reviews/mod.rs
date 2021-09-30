mod tags;

pub use tags::TagsMap;

use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;

use serde::Deserialize;

use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    CouldNotReadCsv(#[from] csv::Error),

    #[error("Couldn't parse advisor review tag for question: {0}")]
    CouldntParseTag(String),
}

type AdvisorReviewExtras = serde_json::Value;

#[derive(Clone, Deserialize)]
struct AdvisorReviewRow {
    id: u32,

    triplet_id: String,

    #[serde(alias = "Idea Title")]
    title: String,

    #[serde(alias = "Idea URL")]
    url: String,

    proposal_id: u32,

    #[serde(alias = "Question")]
    question: String,

    question_id: u32,

    #[serde(alias = "Rating Given")]
    rating: u32,

    #[serde(alias = "Assessor")]
    assessor: String,

    #[serde(alias = "Assessment Note")]
    note: String,

    #[serde(flatten)]
    extras: AdvisorReviewExtras,
}

impl AdvisorReviewRow {
    fn as_advisor_review(&self, tags_map: &TagsMap) -> Result<AdvisorReview, Error> {
        Ok(AdvisorReview {
            id: self.id as i32,
            proposal_id: self.proposal_id as i32,
            rating_given: self.rating as i32,
            assessor: self.assessor.clone(),
            note: self.note.clone(),
            tag: tags_map
                .str_to_tag(&self.question)
                .ok_or_else(|| Error::CouldntParseTag(self.question.clone()))?,
        })
    }
}

pub fn read_vca_reviews_aggregated_file(
    filepath: &Path,
    tags_map: &TagsMap,
) -> Result<Vec<AdvisorReview>, Error> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(filepath)?;

    let mut res: Vec<AdvisorReview> = Vec::new();
    for entry in csv_reader.deserialize() {
        let value: AdvisorReviewRow = entry?;
        res.push(value.as_advisor_review(tags_map)?);
    }
    Ok(res)
}
