use serde::{Deserialize, Serialize};

use super::event::{objective::ObjectiveSummary, proposal::ProposalSummary, EventSummary};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SearchTable {
    Events,
    Objectives,
    Proposals,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SearchColumn {
    Title,
    Type,
    Desc,
    Author,
    Funds,
}

impl ToString for SearchColumn {
    fn to_string(&self) -> String {
        match self {
            SearchColumn::Title => "title".to_string(),
            SearchColumn::Type => "type".to_string(),
            SearchColumn::Desc => "description".to_string(),
            SearchColumn::Author => "author".to_string(),
            SearchColumn::Funds => "funds".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]

pub struct SearchConstraint {
    pub column: SearchColumn,
    pub search: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SearchOrderBy {
    pub column: SearchColumn,
    #[serde(default)]
    pub descending: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SearchQuery {
    pub table: SearchTable,
    #[serde(default)]
    pub filter: Vec<SearchConstraint>,
    #[serde(default)]
    pub order_by: Vec<SearchOrderBy>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ValueResults {
    Events(Vec<EventSummary>),
    Objectives(Vec<ObjectiveSummary>),
    Proposals(Vec<ProposalSummary>),
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct SearchResult {
    pub total: u32,
    pub results: ValueResults,
}

#[cfg(test)]
mod tests {
    use crate::types::event::objective::{ObjectiveId, ObjectiveType};

    use super::*;
    use serde_json::json;

    #[test]
    fn search_query_json_test() {
        assert_eq!(
            SearchQuery {
                table: SearchTable::Objectives,
                filter: vec![
                    SearchConstraint {
                        column: SearchColumn::Title,
                        search: "search 1".to_string(),
                    },
                    SearchConstraint {
                        column: SearchColumn::Type,
                        search: "search 2".to_string(),
                    }
                ],
                order_by: vec![SearchOrderBy {
                    column: SearchColumn::Title,
                    descending: false,
                }],
            },
            serde_json::from_value(json!(
                {
                    "table": "objectives",
                    "filter": [
                        {
                            "column": "title",
                            "search": "search 1"
                        },
                        {
                            "column": "type",
                            "search": "search 2"
                        }
                    ],
                    "order_by": [
                        {
                            "column": "title",
                            "descending": false
                        }
                    ]
                }
            ))
            .unwrap()
        );

        assert_eq!(
            SearchQuery {
                table: SearchTable::Objectives,
                filter: vec![],
                order_by: vec![SearchOrderBy {
                    column: SearchColumn::Title,
                    descending: false,
                }],
            },
            serde_json::from_value(json!(
                {
                    "table": "objectives",
                    "order_by": [
                        {
                            "column": "title",
                        }
                    ]
                }
            ))
            .unwrap()
        );
    }

    #[test]
    fn search_results_json_test() {
        let search_result = SearchResult {
            total: 1,
            results: ValueResults::Objectives(vec![ObjectiveSummary {
                id: ObjectiveId(1),
                objective_type: ObjectiveType {
                    id: "catalyst-native".to_string(),
                    description: "catalyst native type".to_string(),
                },
                title: "objective 1".to_string(),
            }]),
        };

        let json = serde_json::to_value(search_result).unwrap();
        assert_eq!(
            json,
            json!({
                "total": 1,
                "results": [
                        {
                            "id": 1,
                            "type": {
                                "id": "catalyst-native",
                                "description": "catalyst native type"

                            },
                            "title": "objective 1"
                        }
                    ]

            })
        )
    }
}
