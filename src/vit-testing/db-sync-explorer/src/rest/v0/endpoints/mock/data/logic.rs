use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;

pub async fn get_db_sync_content(context: SharedContext) -> Result<String, HandleError> {
    Ok(context
        .read()
        .await
        .get_mock_data_provider()?
        .db_sync_content()?)
}
