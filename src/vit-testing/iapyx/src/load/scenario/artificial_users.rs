use crate::backend::VitRestError;
use crate::backend::VitStationRestClient;
use crate::backend::WalletNodeRestClient;
use crate::load::config::{ArtificialUserLoadConfig, ArtificialUserRequestType as RequestType};
use crate::load::request_generators::AccountRequestGen;
use crate::load::request_generators::ArtificialUserRequestGen;
use crate::load::request_generators::BatchWalletRequestGen;
use crate::load::ServicingStationRequestGen;
use jortestkit::measurement::EfficiencyBenchmarkFinish;
use thiserror::Error;

pub struct ArtificialUserLoad {
    config: ArtificialUserLoadConfig,
}

impl ArtificialUserLoad {
    pub fn new(config: ArtificialUserLoadConfig) -> Self {
        Self { config }
    }

    pub fn start(self) -> Result<Vec<EfficiencyBenchmarkFinish>, Error> {
        let measurement_name = "artificial user load";

        let vit_client = VitStationRestClient::new(self.config.vote.address.clone());
        let node_client = WalletNodeRestClient::new(
            self.config.vote.address.clone(),
            self.config.vote.rest_settings(),
        );
        let transactions = BatchWalletRequestGen::new(
            self.config.vote.build_multi_controller()?,
            self.config.vote.batch_size,
            self.config.vote.use_https,
        );
        let account = AccountRequestGen::new(
            self.config.vote.build_multi_controller()?.into(),
            node_client,
        );
        let fund = ServicingStationRequestGen::new_fund(vit_client.clone());
        let challenges = ServicingStationRequestGen::new_challenges(vit_client.clone());
        let proposal =
            ServicingStationRequestGen::new_proposal(vit_client.clone(), vit_client.proposals()?);
        let proposals = ServicingStationRequestGen::new_proposals(vit_client.clone());

        let request_generators = vec![
            (
                ArtificialUserRequestGen::new_static(fund, RequestType::Fund),
                self.config.fund.clone(),
                "fund request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_static(challenges, RequestType::Challenges),
                self.config.challenges.clone(),
                "challenge request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_static(proposal, RequestType::Proposal),
                self.config.proposal.clone(),
                "proposal request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_static(proposals, RequestType::Proposals),
                self.config.proposals.clone(),
                "proposals request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_account(account),
                self.config.account.clone(),
                "account request".to_string(),
            ),
            (
                ArtificialUserRequestGen::new_node(transactions),
                self.config.vote.config.clone(),
                "proposals request".to_string(),
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("configuration error")]
    LoadConfigError(#[from] crate::load::config::NodeLoadConfigError),
    #[error("configuration error")]
    ServicingConfigError(#[from] crate::load::config::ServicingStationConfigError),
    #[error("rest error")]
    RestError(#[from] VitRestError),
}
