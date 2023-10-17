//! Shared state used by all endpoints.
//!
use crate::{cli::Error, settings::RETRY_AFTER_DELAY_SECONDS_DEFAULT};
use event_db::{queries::EventDbQueries, EventDB};
use std::sync::Arc;

#[cfg(feature = "jorm-mock")]
pub mod jorm_mock;

pub struct State {
    //db_url: Option<String>,
    /// This can be None, or a handle to the DB.
    /// If the DB fails, it can be set to None.
    /// If its None, an attempt to get it will try and connect to the DB.
    /// This is Private, it needs to be accessed with a function.
    //event_db_handle: Arc<ArcSwap<Option<dyn EventDbQueries>>>, // Private need to get it with a function.
    pub event_db: Arc<dyn EventDbQueries>, // This needs to be obsoleted, we want the DB to be able to be down.
    pub delay_seconds: u64,
    #[cfg(feature = "jorm-mock")]
    pub jorm: std::sync::Mutex<jorm_mock::JormState>,
}

impl State {
    pub async fn new(
        database_url: Option<String>,
        delay_seconds: Option<u64>,
    ) -> Result<Self, Error> {
        let delay_seconds: u64 = delay_seconds.unwrap_or(RETRY_AFTER_DELAY_SECONDS_DEFAULT);
        //
        // Get a connection to the Database.
        let db = match database_url.clone() {
            Some(url) => EventDB::new(Some(url.as_str())).await?,
            None => EventDB::new(None).await?,
        };

        #[cfg(feature = "jorm-mock")]
        let jorm = jorm_mock::JormState::new(*crate::settings::JORM_CLEANUP_TIMEOUT);

        let state = Self {
            //db_url: database_url,
            //event_db_handle: Arc::new(RwLock::new(None)),
            event_db: Arc::new(db),
            delay_seconds,
            #[cfg(feature = "jorm-mock")]
            jorm: std::sync::Mutex::new(jorm),
        };

        // We don't care if this succeeds or not.
        // We just try our best to connect to the event DB.
        //let _ = state.event_db().await;

        Ok(state)
    }

    /*
    pub async fn event_db(&self) -> Option<Arc<dyn EventDbQueries>> {


    }
    */
}
