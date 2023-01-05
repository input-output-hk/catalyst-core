use crate::load::config::{ServicingStationLoadConfig, ServicingStationRequestType as RequestType};
use crate::load::ServicingStationRequestGen;
use jortestkit::measurement::EfficiencyBenchmarkFinish;
use thiserror::Error;
use valgrind::VitStationRestClient;

/// Servicing station load scenario. Basically responsible for generating load on static resources.
/// All calls are read only, therefore logic is simple and no need to track state of state.
pub struct ServicingStationLoad {
    config: ServicingStationLoadConfig,
}

impl ServicingStationLoad {
    /// Creates new load scenario object
    pub fn new(config: ServicingStationLoadConfig) -> Self {
        Self { config }
    }

    /// Start scenario
    ///
    /// # Errors
    ///
    /// On any connectivity issues or setup problem
    ///
    pub fn start(self) -> Result<Vec<EfficiencyBenchmarkFinish>, Error> {
        let measurement_name = "servicing station load";

        let vit_client = VitStationRestClient::new(self.config.address.clone());
        let request_generators = vec![
            (
                ServicingStationRequestGen::new_fund(vit_client.clone()),
                self.config.get(RequestType::Fund)?,
                "fund request".to_string(),
            ),
            (
                ServicingStationRequestGen::new_challenges(vit_client.clone()),
                self.config.get(RequestType::Challenges)?,
                "challenge request".to_string(),
            ),
            (
                ServicingStationRequestGen::new_proposal(
                    vit_client.clone(),
                    vit_client.proposals()?,
                ),
                self.config.get(RequestType::Proposal)?,
                "proposal request".to_string(),
            ),
            (
                ServicingStationRequestGen::new_proposals(vit_client),
                self.config.get(RequestType::Proposals)?,
                "proposals request".to_string(),
            ),
        ];

        let stats = jortestkit::load::start_multi_sync(request_generators);

        if let Some(threshold) = self.config.criterion {
            return Ok(stats
                .iter()
                .map(|x| {
                    x.print_summary(measurement_name);
                    x.measure(measurement_name, threshold.into())
                })
                .collect());
        }
        Ok(vec![])
    }
}

/// Servicing station errors
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("configuration error")]
    ConfigurationError(#[from] crate::load::config::ServicingStationConfigError),
    /// Rest api errors
    #[error("rest error")]
    RestError(#[from] valgrind::VitStationRestError),
}
