use super::schemas::ServiceVersion;
use crate::v0::{context::SharedContext, errors::HandleError};
pub async fn service_version(context: SharedContext) -> Result<ServiceVersion, HandleError> {
    let service_version = context.read().await.versioning.clone();
    Ok(ServiceVersion { service_version })
}
