use serde::{Deserialize, Serialize};

use crate::db::models::{challenges::Challenge, proposals::FullProposalInfo};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchQuery {
    #[serde(flatten)]
    pub query: SearchCountQuery,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchCountQuery {
    pub table: Table,
    #[serde(default)]
    pub filter: Vec<Constraint>,
    #[serde(default)]
    pub order_by: Vec<OrderBy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum Constraint {
    Text {
        search: String,
        column: Column,
    },
    Range {
        lower: Option<i64>,
        upper: Option<i64>,
        column: Column,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrderBy {
    Column {
        column: Column,
        #[serde(default)]
        descending: bool,
    },
    Random,
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
    ImpactScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        from_value::<SearchQuery>(json!({"table": "proposals"})).unwrap();
        from_value::<SearchCountQuery>(json!({"table": "proposals"})).unwrap();
    }
}
