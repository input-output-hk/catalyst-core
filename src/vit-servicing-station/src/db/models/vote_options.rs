use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct VoteOptions(pub HashMap<String, u8>);

impl VoteOptions {
    pub fn parse_coma_separated_value(csv: &str) -> VoteOptions {
        VoteOptions(csv.split(',').map(str::to_string).zip(0..).collect())
    }
}
