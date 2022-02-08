mod controller;
mod data;
mod monitor;

pub use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
pub use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;
pub use vit_servicing_station_tests::common::{
    clients::RestClient, startup::server::BootstrapCommandBuilder,
};

pub use controller::{
    VitStationController, VitStationSettings, STORAGE, VIT_CONFIG, VIT_STATION_LOG,
};
pub use data::DbGenerator;
pub use monitor::VitStationMonitorController;
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
