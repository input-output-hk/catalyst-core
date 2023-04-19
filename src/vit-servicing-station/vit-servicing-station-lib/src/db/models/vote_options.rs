use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type VoteOptionsMap = HashMap<String, u8>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct VoteOptions(pub VoteOptionsMap);

impl VoteOptions {
    pub fn parse_coma_separated_value(csv: &str) -> VoteOptions {
        VoteOptions(csv.split(',').map(str::to_string).zip(0..).collect())
    }

    pub fn as_csv_string(&self) -> String {
        self.0
            .iter()
            .sorted_by_key(|(_, &i)| i)
            .map(|(v, _)| v)
            .join(",")
    }

    pub fn from_vec_string(values: Vec<String>) -> VoteOptions {
        VoteOptions(values.into_iter().zip(0..).collect())
    }

    pub fn to_vec_string(&self) -> Vec<String> {
        self.0
            .iter()
            .sorted_by_key(|(_, &i)| i)
            .map(|(v, _)| v.clone())
            .collect()
    }
}
