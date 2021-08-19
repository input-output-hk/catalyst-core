use crate::db::models::community_advisors_reviews::AdvisorReview;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct GroupedReviews(pub HashMap<String, Vec<AdvisorReview>>);
