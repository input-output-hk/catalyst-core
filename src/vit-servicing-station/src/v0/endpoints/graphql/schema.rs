use async_graphql::*;

pub struct Query;

pub struct ProposalData(serde_json::Value);

#[Object]
impl ProposalData {
    async fn value(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }
}

#[Object]
impl Query {
    #[field(desc = "Proposal information")]
    async fn proposal_information<'ctx>(&self, context: &'ctx Context<'_>) -> ProposalData {
        let data = serde_json::from_str(r#"{"msg" : "hello graphql"}"#).unwrap();
        ProposalData(data)
    }
}
