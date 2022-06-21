mod harvester;
mod monitor;
mod settings;

pub use harvester::Harvester;
pub use monitor::start;
pub use settings::Settings;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Rest(#[from] jormungandr_automation::jormungandr::RestError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
