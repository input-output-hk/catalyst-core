use crate::cli::Error;
use event_db::queries::EventDbQueries;
use std::sync::Arc;

#[cfg(feature = "jorm-mock")]
pub mod jorm_mock;

pub struct State {
    pub event_db: Arc<dyn EventDbQueries>,
    #[cfg(feature = "jorm-mock")]
    pub jorm: std::sync::Mutex<jorm_mock::JormState>,
}

impl State {
    pub async fn new(database_url: Option<String>) -> Result<Self, Error> {
        let event_db = if let Some(url) = database_url {
            Arc::new(event_db::establish_connection(Some(url.as_str())).await?)
        } else {
            Arc::new(event_db::establish_connection(None).await?)
        };

        #[cfg(feature = "jorm-mock")]
        let jorm = {
            if let Ok(arg) = std::env::var("JORM_CLEANUP_TIMEOUT") {
                let duration = arg.parse::<u64>().map_err(|e| {
                    Error::Service(crate::service::Error::CannotRunService(e.to_string()))
                })?;
                jorm_mock::JormState::new(std::time::Duration::from_secs(duration * 60))
            } else {
                jorm_mock::JormState::new(jorm_mock::JormState::CLEANUP_TIMEOUT)
            }
        };

        Ok(Self {
            event_db,
            #[cfg(feature = "jorm-mock")]
            jorm: std::sync::Mutex::new(jorm),
        })
    }
}
