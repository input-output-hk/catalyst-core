use crate::load::multi_controller::MultiControllerError;
use crate::load::request_generators::RequestGenError;
use crate::load::request_generators::{BatchWalletRequestGen, WalletRequestGen};
use crate::load::status_provider::VoteStatusProvider;
use crate::NodeLoadConfig;
use crate::NodeLoadError;
use jortestkit::measurement::EfficiencyBenchmarkFinish;
use thiserror::Error;

/// Load scenario for node only calls.
/// It uses `WalletRequestGen` or `BatchWalletRequestGen` based on configuration of requests per step
/// setting.
pub struct NodeLoad {
    config: NodeLoadConfig,
}

impl NodeLoad {
    /// Creates new object
    #[must_use]
    pub fn new(config: NodeLoadConfig) -> Self {
        Self { config }
    }

    /// Starts scenario
    ///
    /// # Errors
    ///
    /// On any error related with setup or lack of connectivity
    ///
    pub fn start(self) -> Result<Option<EfficiencyBenchmarkFinish>, NodeLoadError> {
        let backend = self.config.address.clone();

        let mut multicontroller = self.config.build_multi_controller()?;

        if self.config.reuse_accounts_early {
            multicontroller.update_wallets_state()?;
        }

        let measurement_name = "iapyx load test";

        let stats = if self.config.batch_size > 1 {
            jortestkit::load::start_async(
                BatchWalletRequestGen::new(
                    multicontroller,
                    self.config.batch_size,
                    self.config.use_v1,
                    self.config.reuse_accounts_lazy,
                    &self.config.voting_group,
                )?,
                VoteStatusProvider::new(backend, self.config.debug)?,
                self.config.config,
                measurement_name,
            )
        } else {
            jortestkit::load::start_sync(
                WalletRequestGen::new(
                    multicontroller,
                    self.config.reuse_accounts_lazy,
                    &self.config.voting_group,
                )?,
                self.config.config,
                measurement_name,
            )
        };

        stats.print_summary(measurement_name);

        if let Some(threshold) = self.config.criterion {
            return Ok(Some(stats.measure(measurement_name, threshold.into())));
        }
        Ok(None)
    }
}

/// Errors for node load
#[derive(Error, Debug)]
pub enum Error {
    /// Controller errors
    #[error("internal error")]
    MultiControllerError(#[from] MultiControllerError),
    /// Requests generator error
    #[error("request gen error")]
    RequestGen(#[from] RequestGenError),
    /// Status providers error
    #[error("request gen error")]
    StatusProvider(#[from] crate::load::StatusProviderError),
}
