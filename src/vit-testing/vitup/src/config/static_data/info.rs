use serde::{Deserialize, Serialize};

use crate::config::Role;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FundInfo {
    #[serde(default = "default_goals")]
    pub goals: Vec<String>,
    #[serde(default = "default_result_url")]
    pub results_url: String,
    #[serde(default = "default_survey_url")]
    pub survey_url: String,
    pub fund_name: String,
    pub fund_id: i32,
    pub groups: Vec<String>,
}

impl From<i32> for FundInfo {
    fn from(fund_id: i32) -> Self {
        Self {
            results_url: default_result_url(),
            survey_url: default_survey_url(),
            goals: default_goals(),
            fund_id,
            fund_name: format!("Fund{}", fund_id),
            groups: default_groups(),
        }
    }
}

fn default_result_url() -> String {
    "https://catalyst.domain/survey".to_string()
}

fn default_survey_url() -> String {
    "https://catalyst.domain/result".to_string()
}

fn default_goals() -> Vec<String> {
    vec![
        "first Goal".to_string(),
        "second Goal".to_string(),
        "third Goal".to_string(),
    ]
}

fn default_groups() -> Vec<String> {
    vec![Role::Voter.to_string(), Role::Representative.to_string()]
}
