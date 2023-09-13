use poem_openapi::Object;
use serde_json::Value;

/// Details about a Type of review.
#[derive(Object)]
pub struct ReviewType {
    /// The Unique ID for this review type.
    id: i32,

    /// The unique name for the review type.
    name: String,

    /// Description about what the review type is.
    #[oai(skip_serializing_if_is_none = true)]
    description: Option<String>,

    /// The inclusive Minimum value for the reviews rating.
    /// By definition, lower value ratings are considered lower ratings.
    /// Therefore this field represents the lowest possible rating.
    min: i32,

    /// The inclusive Maximum value for the reviews rating.
    /// By definition, higher value ratings are considered higher ratings.
    /// Therefore this field represents the highest possible rating.
    max: i32,

    /// Optional sequential list of mapped named values for rating scores.
    /// * If not present, the rating score is numeric.
    /// * If present:
    ///  * all possible rating scores must be represented with mapped names and the rating is represented by the value in the map.
    ///  * The lowest numbered score comes first in the array.
    ///  * The array is sequential with no gaps.
    map: Vec<Value>,

    /// Does the Review Type include a note?
    /// * Null - *Optional*, may or may not include a note.
    /// * False - **MUST NOT** include a note.
    /// * True - **MUST** include a note.
    #[oai(skip_serializing_if_is_none = true)]
    note: Option<bool>,

    /// The reviewer group who can create this review type.
    #[oai(skip_serializing_if_is_none = true)]
    group: Option<String>,
}

impl From<event_db::types::reviews::ReviewType> for ReviewType {
    fn from(value: event_db::types::reviews::ReviewType) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            min: value.min,
            max: value.max,
            map: value.map,
            note: value.note,
            group: value.group,
        }
    }
}

/// Individual Rating.
#[derive(Object)]
pub struct Rating {
    /// The review type being rated. Maps to the ReviewType id.
    review_type: i32,

    /// Score given to this rating.
    /// Will be bounded by the `min` and `max` of the ReviewType.
    score: i32,

    /// Reason why this rating was given.
    /// If NO reason was given, this field is omitted.
    #[oai(skip_serializing_if_is_none = true)]
    note: &'a Option<String>,
}

impl From<event_db::types::reviews::Rating> for Rating {
    fn from(value: event_db::types::reviews::Rating) -> Self {
        Self {
            review_type: value.review_type,
            score: value.score,
            note: value.note,
        }
    }
}

/// Review of a Proposal by a Community Advisor.
#[derive(Object)]
pub struct AdvisorReview {
    /// Anonymized Assessor identity.
    /// All reviews by the same Assessor will have the same identity string.
    assessor: String,

    /// List of review ratings given by this reviewer.
    ratings: Vec<Rating>,
}

impl From<event_db::types::reviews::AdvisorReview> for AdvisorReview {
    fn from(value: event_db::types::reviews::AdvisorReview) -> Self {
        Self {
            assessor: value.assessor,
            ratings: value.ratings.into_iter().map(Into::into).collect(),
        }
    }
}
