use crate::load::config::{ArtificialUserLoadConfig, ArtificialUserRequestType as RequestType};
use crate::load::request_generators::AccountRequestGen;
use crate::load::request_generators::ArtificialUserRequestGen;
use crate::load::request_generators::BatchWalletRequestGen;
use crate::load::request_generators::SettingsRequestGen;
use crate::load::ServicingStationRequestGen;
use jortestkit::measurement::EfficiencyBenchmarkFinish;
use thiserror::Error;
use valgrind::{VitStationRestClient, VitStationRestError};

/// Struct responsible to simulate real user calls send to Catalyst Backend.
/// It uses combination of request generators with given proportions defined in settings.
/// The idea is that if properly configured it can simulate REST API calls which will be generated
/// from user using mobile apps. Usually users can be divided in 3 categories:
/// - Browser - not voting at all, just generating load on static data
/// - Fast voter - many votes, small amount of static data request
/// - Indecisive voter - small amount of votes, many requests to static data
/// - All in voter - big batches of votes
pub struct ArtificialUserLoad {
    config: ArtificialUserLoadConfig,
}

impl ArtificialUserLoad {
    /// Creates new object based on configuration
    #[must_use]
    pub fn new(config: ArtificialUserLoadConfig) -> Self {
        Self { config }
    }

    /// Starts load based on configuration
    ///
    /// # Errors
    ///
    /// On any load related issue like lack of connectivity or data preparation problem
    pub fn start(self) -> Result<Vec<EfficiencyBenchmarkFinish>, Error> {
        let measurement_name = "artificial user load";

        let vit_client = VitStationRestClient::new(self.config.vote.address.clone());
        let mut multi_controller = self.config.vote.build_multi_controller()?;

        if self.config.vote.reuse_accounts_early {
            multi_controller.update_wallets_state()?;
        }

        let node_client = multi_controller.backend().node_client();

        let transactions = BatchWalletRequestGen::new(
            multi_controller,
            self.config.vote.batch_size,
            self.config.vote.use_https,
            self.config.vote.reuse_accounts_lazy,
            &self.config.vote.voting_group,
        )?;
        let account = AccountRequestGen::new(
            self.config.vote.build_multi_controller()?.into(),
            node_client.clone(),
        );
        let settings = SettingsRequestGen::new(node_client);
        let challenge =
            ServicingStationRequestGen::new_challenge(vit_client.clone(), vit_client.challenges()?);
        let fund = ServicingStationRequestGen::new_fund(vit_client.clone());
        let challenges = ServicingStationRequestGen::new_challenges(vit_client.clone());
        let proposal =
            ServicingStationRequestGen::new_proposal(vit_client.clone(), vit_client.proposals()?);

        let request_generators = vec![
            (
                ArtificialUserRequestGen::new_static(proposal, RequestType::Proposal),
                self.config.proposal.clone(),
                "proposal request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_static(fund, RequestType::Fund),
                self.config.fund.clone(),
                "fund request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_static(challenge, RequestType::Challenge),
                self.config.challenge.clone(),
                "challenge request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_static(challenges, RequestType::Challenges),
                self.config.challenges.clone(),
                "challenges request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_settings(settings),
                self.config.settings.clone(),
                "settings request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_account(account),
                self.config.account.clone(),
                "account request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_node(transactions),
                self.config.vote.config.clone(),
                "vote request".to_string(),
            ),
        ];

        let stats = jortestkit::load::start_multi_sync(request_generators);

        if let Some(threshold) = self.config.vote.criterion {
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

/// Generator errors
#[derive(Error, Debug)]
pub enum Error {
    /// Loading configuration
    #[error("configuration error")]
    LoadConfig(#[from] crate::load::config::NodeLoadConfigError),
    /// Building configuration
    #[error("configuration error")]
    ServicingConfig(#[from] crate::load::config::ServicingStationConfigError),
    /// Rest api errors
    #[error("rest error")]
    Rest(#[from] VitStationRestError),
    /// Controller errors
    #[error("controller error")]
    MultiController(#[from] crate::load::MultiControllerError),
}
