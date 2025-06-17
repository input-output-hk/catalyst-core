use std::fmt::Display;
use super::{event::EventSummary, objective::ObjectiveSummary, proposal::ProposalSummary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchTable {
    Events,
    Objectives,
    Proposals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchColumn {
    Title,
    Type,
    Description,
    Author,
    Funds,
}

impl Display for SearchColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SearchColumn::Title => "title".to_string(),
            SearchColumn::Type => "type".to_string(),
            SearchColumn::Description => "description".to_string(),
            SearchColumn::Author => "author".to_string(),
            SearchColumn::Funds => "funds".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct SearchConstraint {
    pub column: SearchColumn,
    pub search: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOrderBy {
    pub column: SearchColumn,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub table: SearchTable,
    pub filter: Vec<SearchConstraint>,
    pub order_by: Vec<SearchOrderBy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueResults {
    Events(Vec<EventSummary>),
    Objectives(Vec<ObjectiveSummary>),
    Proposals(Vec<ProposalSummary>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub total: i64,
    pub results: Option<ValueResults>,
}
