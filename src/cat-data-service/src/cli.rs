use crate::{logger, service, settings::Settings};
use clap::Parser;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ServiceError(#[from] service::Error),
    #[error(transparent)]
    LoggerError(#[from] SetGlobalDefaultError),
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

                service::run_service(&settings.address).await?;
                Ok(())
            }
        }
    }
}
