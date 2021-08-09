mod tags;

pub use tags::TagsMap;

use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;

use calamine::{open_workbook, Reader, Xlsx};
use serde::Deserialize;

use std::collections::HashMap;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    CouldNotReadExcel(#[from] calamine::Error),

    #[error(transparent)]
    Xls(#[from] calamine::XlsxError),

    #[error(transparent)]
    CouldNotDeserialize(#[from] calamine::DeError),

    #[error("Couldn't find workbook {0}")]
    CouldNotFindWorkbook(String),

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
    worksheet: &str,
    tags_map: &TagsMap,
) -> Result<Vec<AdvisorReview>, Error> {
    let mut workbook: Xlsx<_> = open_workbook(filepath)?;
    let range = workbook
        .worksheet_range(worksheet)
        .ok_or_else(|| Error::CouldNotFindWorkbook(worksheet.to_string()))??;
    let mut res: Vec<AdvisorReview> = Vec::new();
    for entry in range.deserialize()? {
        let value: AdvisorReviewRow = entry?;
        res.push(value.as_advisor_review(tags_map)?);
    }
    Ok(res)
}
