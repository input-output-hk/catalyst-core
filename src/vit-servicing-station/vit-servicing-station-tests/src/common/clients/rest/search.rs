use vit_servicing_station_lib::db::models::proposals::ChallengeType;
use vit_servicing_station_lib::v0::endpoints::search::requests::{
    Column, Constraint, OrderBy, SearchCountQuery, SearchQuery, Table,
};

pub struct SearchRequestBuilder {
    query: SearchQuery,
}

#[allow(clippy::from_over_into)]
impl Into<SearchQuery> for SearchRequestBuilder {
    fn into(self) -> SearchQuery {
        self.query
    }
}

impl Default for SearchRequestBuilder {
    fn default() -> Self {
        Self {
            query: SearchQuery {
                query: SearchCountQuery {
                    table: Table::Challenges,
                    filter: vec![],
                    order_by: vec![],
                },
                limit: None,
                offset: None,
            },
        }
    }
}

impl SearchRequestBuilder {
    pub fn on_challenges(self) -> Self {
        self.on(Table::Challenges)
    }

    pub fn on_proposals(self) -> Self {
        self.on(Table::Proposals)
    }

    pub fn on(mut self, table: Table) -> Self {
        self.query.query.table = table;
        self
    }

    pub fn by_funds_exact(self, funds: i64) -> Self {
        self.by_funds(Some(funds), Some(funds))
    }

    pub fn by_funds(self, lower: Option<i64>, upper: Option<i64>) -> Self {
        self.by_range(lower, upper, Column::Funds)
    }

    pub fn by_impact_score(self, impact_score: i64) -> Self {
        self.by_range(Some(impact_score), Some(impact_score), Column::ImpactScore)
    }

    pub fn by_author(self, author: impl Into<String>) -> Self {
        self.by_text(author, Column::Author)
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.query.offset = Some(offset);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.query.limit = Some(limit);
        self
    }

    pub fn by_body(self, body: impl Into<String>) -> Self {
        self.by_text(body, Column::Desc)
    }

    pub fn by_title(self, title: impl Into<String>) -> Self {
        self.by_text(title.into(), Column::Title)
    }

    pub fn by_type(self, challenge_type: &ChallengeType) -> Self {
        self.by_text(challenge_type.to_string(), Column::Type)
    }

    pub fn by_text(mut self, phrase: impl Into<String>, column: Column) -> Self {
        self.query.query.filter.push(Constraint::Text {
            search: phrase.into(),
            column,
        });
        self
    }

    pub fn by_range(mut self, lower: Option<i64>, upper: Option<i64>, column: Column) -> Self {
        self.query.query.filter.push(Constraint::Range {
            upper,
            lower,
            column,
        });
        self
    }

    pub fn order_by_title(self) -> Self {
        self.order_by_desc(Column::Title)
    }

    pub fn order_by_asc(mut self, column: Column) -> Self {
        self.query.query.order_by.push(OrderBy::Column {
            column,
            descending: false,
        });
        self
    }

    pub fn order_by_desc(mut self, column: Column) -> Self {
        self.query.query.order_by.push(OrderBy::Column {
            column,
            descending: true,
        });
        self
    }

    pub fn order_by_random(mut self) -> Self {
        self.query.query.order_by.push(OrderBy::Random);
        self
    }
}
