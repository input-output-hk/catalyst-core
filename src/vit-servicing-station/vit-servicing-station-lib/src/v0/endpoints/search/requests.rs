use serde::{Deserialize, Serialize};

use crate::db::models::{challenges::Challenge, proposals::FullProposalInfo};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Query {
    pub table: Table,
    #[serde(default)]
    pub filter: Vec<Constraint>,
    #[serde(default)]
    pub order_by: Vec<OrderBy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Constraint {
    pub search: String,
    pub column: Column,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct OrderBy {
    pub column: Column,
    #[serde(default)]
    pub descending: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Table {
    Challenges,
    Proposals,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Column {
    Title,
    Type,
    Desc,
    Author,
    Funds,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // should serialize as if it is either a `Vec<Challenge>` or `Vec<FullProposalInfo>`
pub enum SearchResponse {
    Challenge(Vec<Challenge>),
    Proposal(Vec<FullProposalInfo>),
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json, to_string};

    use crate::db::models::proposals::test::get_test_proposal;

    use super::*;

    #[test]
    fn response_serializes_as_vec() {
        let response = SearchResponse::Proposal(vec![get_test_proposal("asdf")]);
        let s = to_string(&response).unwrap();
        assert!(s.starts_with('['));
        assert!(s.ends_with(']'));
    }

    #[test]
    fn filters_and_orders_are_optional() {
        from_value::<Query>(json!({"table": "proposals"})).unwrap();
    }
}
