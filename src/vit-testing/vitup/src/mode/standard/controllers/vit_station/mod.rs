pub mod controller;
mod data;

pub use vit_servicing_station_tests::common::{
    clients::RestClient,
    data::{
        ArbitraryValidVotingTemplateGenerator, ExternalValidVotingTemplateGenerator,
        ValidVotePlanGenerator, ValidVotePlanParameters, ValidVotingTemplateGenerator,
    },
    startup::{db::DbBuilder, server::BootstrapCommandBuilder},
};

pub use vit_servicing_station_lib::server::settings::dump_settings_to_file;

pub use data::{generate_database, generate_random_database, DbGenerator, Error as DataError};
use std::time::Duration;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(custom_debug::Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    RestError(#[from] vit_servicing_station_tests::common::clients::RestError),
    #[error("port already binded: {0}")]
    PortAlreadyBinded(u16),
    #[error("no vit station defined in settings")]
    NoVitStationDefinedInSettings,
    #[error("no vit station archiver defined in settings")]
    NoVitStationArchiverDefinedInSettings,
    #[error("fragment logs in an invalid format")]
    InvalidFragmentLogs(#[source] serde_json::Error),
    #[error("node '{alias}' failed to start after {} s", .duration.as_secs())]
    NodeFailedToBootstrap {
        alias: String,
        duration: Duration,
        #[debug(skip)]
        logs: Vec<String>,
    },
    #[error("node '{alias}' failed to shutdown, message: {message}")]
    NodeFailedToShutdown {
        alias: String,
        message: String,
        #[debug(skip)]
        logs: Vec<String>,
    },
}
