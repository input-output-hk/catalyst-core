use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SearchTable {
    Events,
    Objectives,
    Proposals,
}

impl ToString for SearchTable {
    fn to_string(&self) -> String {
        match self {
            SearchTable::Events => "events".to_string(),
            SearchTable::Objectives => "objectives".to_string(),
            SearchTable::Proposals => "proposals".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
            SearchColumn::Desc => "desc".to_string(),
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

#[cfg(test)]
mod tests {
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
}
