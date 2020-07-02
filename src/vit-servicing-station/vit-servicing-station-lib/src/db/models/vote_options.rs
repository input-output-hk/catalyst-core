use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct VoteOptions(pub HashMap<String, u8>);

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
}
