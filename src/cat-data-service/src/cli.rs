use crate::{logger, service, settings::Settings, state::State};
use clap::Parser;
use std::sync::Arc;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ServiceError(#[from] service::Error),
    #[error(transparent)]
    LoggerError(#[from] SetGlobalDefaultError),
    #[error(transparent)]
    EventDbError(#[from] event_db::Error),
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
                logger::init(settings.log_level)?;

                let state = Arc::new(State::new(settings.database_url).await?);
                service::run_service(&settings.address, state).await?;
                Ok(())
            }
        }
    }
}
