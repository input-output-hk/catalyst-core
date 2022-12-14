use crate::rest::v0::endpoints::mock::control::logic::reset_data;
use crate::rest::v0::endpoints::mock::control::request::ResetRequest;
use crate::rest::v0::errors::HandleError;
use crate::rest::v0::SharedContext;
use crate::MockError;
use warp::{Rejection, Reply};

pub async fn reset(
    reset_request: ResetRequest,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let mut context = context.write().await;
    let mock_config = context
        .config()
        .mock_config()
        .ok_or_else(|| MockError::Internal(
            "cannot retrieve mock configuration".to_string(),
        ))
        .map_err(HandleError::Mock)?;
    let (ledger, new_db_sync_instance) = reset_data(reset_request, mock_config.providers).await?;
    let data_provider = context.get_mock_data_provider_mut()?;

    *data_provider.db_sync_mut() = new_db_sync_instance;
    *data_provider.ledger_mut() = ledger;

    Ok(warp::reply())
}
