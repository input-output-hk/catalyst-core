use crate::db::{models::Proposal, schema::proposals::dsl::proposals};
use crate::v0::context::{ChainData, SharedContext};
use diesel::RunQueryDsl;

pub async fn get_all_proposals(context: SharedContext) -> Vec<Proposal> {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .expect("Error connecting to database");
    // TODO: make a non blocking task for this retrieve
    proposals
        .load::<Proposal>(&db_conn)
        .expect("Error loading proposals")
}

pub async fn get_data_from_id(id: String, context: SharedContext) -> Option<ChainData> {
    context.read().await.static_chain_data.get(&id).cloned()
}

#[cfg(test)]
mod test {
    use crate::v0::context::test::fake_data_context;

    #[tokio::test]
    async fn get_data_from_id() -> Result<(), ()> {
        let (id, json_data, context) = fake_data_context();
        let result = super::get_data_from_id(id.clone(), context).await.unwrap();
        assert_eq!(json_data, result);
        Ok(())
    }
}
