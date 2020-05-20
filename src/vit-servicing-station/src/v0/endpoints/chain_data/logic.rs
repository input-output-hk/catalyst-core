use crate::v0::context::{ChainData, SharedContext};
use serde_json::Value;
use warp::{Rejection, Reply};

pub async fn get_data_from_id(id: String, context: SharedContext) -> Option<ChainData> {
    context
        .read()
        .await
        .static_chain_data
        .get(&id)
        .map(|data| data.clone())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::v0::context::{ChainDataStore, Context};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn get_data_from_id() -> Result<(), ()> {
        // build fake data
        let data = r#"{"foo" : "bar"}"#;
        let json_data: ChainData = serde_json::from_str(data).unwrap();

        let id = String::from("foo");

        // build fake context chain data
        let mut context_data = ChainDataStore::new();
        context_data.insert(id.clone(), json_data.clone());

        let context = Arc::new(RwLock::new(Context::new(context_data)));

        let result = super::get_data_from_id(id.clone(), context).await.unwrap();
        assert_eq!(json_data, result);
        Ok(())
    }
}
