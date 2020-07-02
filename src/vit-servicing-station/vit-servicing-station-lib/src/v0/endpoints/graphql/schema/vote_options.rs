use crate::db::models::vote_options::VoteOptions;
use async_graphql::registry::Registry;
use async_graphql::{ContextSelectionSet, Positioned};
use std::borrow::Cow;

#[async_trait::async_trait]
impl async_graphql::OutputValueType for VoteOptions {
    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<async_graphql::parser::query::Field>,
    ) -> async_graphql::Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.0).unwrap())
    }
}

impl async_graphql::Type for VoteOptions {
    fn type_name() -> Cow<'static, str> {
        Cow::from("VoteOptions")
    }

    fn create_type_info(_registry: &mut Registry) -> String {
        Self::qualified_type_name()
    }
}
