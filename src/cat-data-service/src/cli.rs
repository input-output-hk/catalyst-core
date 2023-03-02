use clap::Parser;

use crate::{service, settings::Settings};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ServiceError(#[from] service::Error),
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
                service::run_service(&settings.address).await?;
                Ok(())
            }
        }
    }
}
