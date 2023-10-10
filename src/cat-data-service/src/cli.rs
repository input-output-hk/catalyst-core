use crate::{
    logger, service,
    settings::{RetryAfterParams, Settings},
    state::State,
};
use clap::Parser;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Service(#[from] service::Error),
    #[error(transparent)]
    EventDb(#[from] event_db::error::Error),
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Cli {
    Run(Settings),
}

impl Cli {
    pub async fn exec(self) -> Result<(), Error> {
        match self {
            Self::Run(settings) => {
                logger::init(settings.log_level).unwrap();

                let state = Arc::new(
                    State::new_with_delay(Some(settings.database_url), settings.delay_seconds)
                        .await?,
                );

                // Check the schema version, if connection to DB timesout, log as warning and
                // continue, otherwise, return Error.
                match state.event_db.schema_version_check().await {
                    Ok(current_ver) => {
                        tracing::info!(schema_version = current_ver, "verified schema version")
                    }
                    Err(e) => {
                        tracing::warn!(error = e.to_string(), "could not verify schema version");
                        // Only return error if it is not a connection timeout
                        if e != event_db::error::Error::ConnectionTimeout {
                            return Err(e.into());
                        }
                    }
                };

                // Initialize the `RETRY_AFTER_DELAY_SECONDS` env var with the
                // initial value stored in settings.
                RetryAfterParams::delay_seconds_set_var(settings.delay_seconds);

                service::run(&settings.address, &settings.metrics_address, state).await?;
                Ok(())
            }
        }
    }
}
