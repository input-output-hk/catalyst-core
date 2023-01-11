use crate::db::Meta;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;

pub async fn get_meta_info(context: SharedContext) -> Result<Vec<Meta>, HandleError> {
    let context = context.read().await;
    let provider = context.provider();
    provider.get_meta_info().await
}
