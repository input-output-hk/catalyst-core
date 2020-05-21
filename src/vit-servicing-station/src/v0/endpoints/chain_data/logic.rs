use crate::v0::context::{ChainData, SharedContext};

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
