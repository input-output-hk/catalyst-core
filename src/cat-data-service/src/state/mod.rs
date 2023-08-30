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
        let jorm = jorm_mock::JormState::new(*crate::settings::JORM_CLEANUP_TIMEOUT);

        Ok(Self {
            event_db,
            #[cfg(feature = "jorm-mock")]
            jorm: std::sync::Mutex::new(jorm),
        })
    }
}
