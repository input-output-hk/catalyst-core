pub mod helpers;
mod reviews;
pub mod utils;

use crate::builders::helpers::VitVotePlanDefBuilder;
pub use crate::builders::helpers::{build_current_fund, build_servicing_station_parameters};
use crate::builders::utils::DeploymentTree;
use crate::config;
use crate::config::{Config, Role};
use crate::mode::standard::{VitController, VitControllerBuilder};
use assert_fs::fixture::ChildPath;
use chain_impl_mockchain::chaintypes::ConsensusVersion;
use chain_impl_mockchain::testing::TestGen;
use chain_impl_mockchain::tokens::identifier::TokenIdentifier;
use chain_impl_mockchain::tokens::minting_policy::MintingPolicy;
use chain_impl_mockchain::value::Value;
use config::Block0Initial::Wallet;
pub use helpers::{
    convert_to_blockchain_date, convert_to_human_date, discover_archive_input_files,
    generate_qr_and_hashes, get_configuration_from_file_url, ArchiveConfError,
    ArchiveConfiguration,
};
use hersir::builder::settings::Blockchain;
use hersir::builder::Node;
use hersir::builder::Topology;
use hersir::config::{CommitteeTemplate, ExplorerTemplate, SessionSettings, WalletTemplate};
use jormungandr_automation::jormungandr::PersistenceMode;
use jormungandr_lib::interfaces::NumberOfSlotsPerEpoch;
use jormungandr_lib::interfaces::SlotDuration;
use jormungandr_lib::interfaces::TokenIdentifier as TokenIdentifierLib;
pub use reviews::ReviewGenerator;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use tracing::{info, span, Level, Span};
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;

pub const LEADER_1: &str = "leader1";
pub const LEADER_2: &str = "leader2";
pub const LEADER_3: &str = "leader3";
pub const FOLLOWER: &str = "follower";

#[derive(Clone)]
pub struct VitBackendSettingsBuilder {
    config: Config,
    session_settings: SessionSettings,
    committee_wallet: String,
    //needed for load tests when we rely on secret keys instead of qrs
    skip_qr_generation: bool,
    span: Span,
}

impl Default for VitBackendSettingsBuilder {
    fn default() -> Self {
        Self {
            committee_wallet: "committee_1".to_owned(),
            skip_qr_generation: false,
            config: Default::default(),
            session_settings: SessionSettings::default(),
            span: span!(Level::INFO, "builder"),
        }
    }
}

impl VitBackendSettingsBuilder {
    pub fn skip_qr_generation(mut self) -> Self {
        self.skip_qr_generation = true;
        self
    }

    pub fn span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn config(mut self, config: &Config) -> Self {
        self.config = config.clone();
        self
    }

    pub fn session_settings(mut self, session_settings: SessionSettings) -> Self {
        self.session_settings = session_settings;
        self
    }

    pub fn build_topology(&self) -> Topology {
        let topology = Topology::default();

        // Leader 1
        let leader_1 = Node::new(LEADER_1);

        // leader 2
        let leader_2 = Node::new(LEADER_2).with_trusted_peer(LEADER_1);

        // leader 3
        let leader_3 = Node::new(LEADER_3)
            .with_trusted_peer(LEADER_1)
            .with_trusted_peer(LEADER_2);

        // passive
        let passive = Node::new(FOLLOWER)
            .with_trusted_peer(LEADER_1)
            .with_trusted_peer(LEADER_2)
            .with_trusted_peer(LEADER_3);

        topology
            .with_node(leader_1)
            .with_node(leader_2)
            .with_node(leader_3)
            .with_node(passive)
    }

    pub fn dump_qrs<P: AsRef<Path>>(
        &self,
        controller: &VitController,
        initials: &HashMap<WalletTemplate, String>,
        child: P,
    ) -> Result<(), Error> {
        let deployment_tree = DeploymentTree::new(child.as_ref());
        let folder = deployment_tree.qr_codes_path();
        std::fs::create_dir_all(&folder)?;

        let defined_wallets = controller.defined_wallets();
        let wallets: Vec<(_, _)> = defined_wallets
            .iter()
            .filter(|(_, x)| {
                !(x.template().alias().is_some()
                    && x.template().alias().unwrap().starts_with("committee"))
                    && x.template().is_generated()
            })
            .map(|(alias, template)| {
                let wallet: thor::Wallet = ((*template).clone()).into();
                (alias, wallet)
            })
            .collect();

        generate_qr_and_hashes(wallets, initials, &self.config, &folder).map_err(Into::into)
    }

    fn write_token<P: AsRef<Path>>(
        &self,
        path: P,
        token_list: &[(Role, TokenIdentifier)],
    ) -> Result<(), Error> {
        let token_list: Vec<(Role, TokenIdentifierLib)> = token_list
            .iter()
            .cloned()
            .map(|(r, t)| (r, t.into()))
            .collect();
        let mut file = std::fs::File::create(&path)?;
        file.write_all(serde_json::to_string(&token_list)?.as_bytes())
            .map_err(Into::into)
    }

    pub fn build(mut self) -> Result<(VitController, ValidVotePlanParameters), Error> {
        let _enter = self.span.enter();

        let mut builder = VitControllerBuilder::new();

        let vote_blockchain_time = convert_to_blockchain_date(&self.config);

        let mut blockchain = Blockchain::default()
            .with_consensus(ConsensusVersion::Bft)
            .with_slots_per_epoch(
                NumberOfSlotsPerEpoch::new(vote_blockchain_time.slots_per_epoch).unwrap(),
            )
            .with_slot_duration(SlotDuration::new(self.config.blockchain.slot_duration).unwrap());

        info!("building topology..");

        builder = builder.topology(self.build_topology());
        blockchain = blockchain
            .with_leader(LEADER_1)
            .with_leader(LEADER_2)
            .with_leader(LEADER_3);

        info!("building blockchain parameters..");

        blockchain = blockchain
            .with_linear_fee(self.config.blockchain.linear_fees.clone())
            .with_tx_max_expiry_epochs(self.config.blockchain.tx_max_expiry_epochs)
            .with_discrimination(chain_addr::Discrimination::Production)
            .with_block_content_max_size(self.config.blockchain.block_content_max_size.into())
            .with_block0_date(self.config.blockchain.block0_date_as_unix());

        if !self.config.blockchain.consensus_leader_ids.is_empty() {
            blockchain = blockchain.with_external_consensus_leader_ids(
                self.config.blockchain.consensus_leader_ids.clone(),
            );
        }

        if !self.config.blockchain.committees.is_empty() {
            blockchain = blockchain.with_committees(self.config.blockchain.committees.clone());
        }

        let committe = CommitteeTemplate::Generated {
            alias: self.committee_wallet.clone(),
            member_pk: None,
            communication_pk: None,
        };

        builder = builder.committee(committe);

        info!("building voting token..");

        let root = self.session_settings.root.path().to_path_buf();
        std::fs::create_dir_all(&root)?;
        let policy = MintingPolicy::new();
        let token_list: Vec<(Role, TokenIdentifier)> = self
            .config
            .data
            .current_fund
            .fund_info
            .groups
            .iter()
            .map(|role| {
                (
                    Role::from_str(role).unwrap(),
                    TokenIdentifier {
                        policy_hash: policy.hash(),
                        token_name: TestGen::token_name(),
                    },
                )
            })
            .collect();

        let tokens_map = |role: &Role| {
            token_list
                .iter()
                .find(|(a, _)| a == role)
                .map(|(_, b)| b.clone().into())
                .unwrap()
        };

        self.write_token(root.join("voting_token.txt"), &token_list)?;

        info!("building initials..");

        self.config.initials.block0.push(Wallet {
            name: self.committee_wallet.clone(),
            funds: 1_000_000,
            pin: "".to_string(),
            role: Default::default(),
        });

        let mut generated_wallet_templates = HashMap::new();

        if self.config.initials.block0.any() {
            builder = builder.wallets(self.config.initials.block0.external_templates(tokens_map));
            generated_wallet_templates = self.config.initials.block0.templates(
                self.config.data.current_fund.voting_power,
                blockchain.discrimination(),
                tokens_map,
            );
            for (wallet, _) in generated_wallet_templates
                .iter()
                .filter(|(x, _)| *x.value() > Value::zero())
            {
                builder = builder.wallet(wallet.clone());
            }
        }
        info!("building direct voteplan..");

        builder = builder.vote_plans(
            VitVotePlanDefBuilder::default()
                .vote_phases(vote_blockchain_time)
                .options(
                    self.config
                        .data
                        .current_fund
                        .options
                        .0
                        .len()
                        .try_into()
                        .map_err(|_| Error::TooManyOptions)?,
                )
                .split_by(255)
                .fund_name(
                    self.config
                        .data
                        .current_fund
                        .fund_info
                        .fund_name
                        .to_string(),
                )
                .committee(self.committee_wallet.clone())
                .private(self.config.vote_plan.private)
                .proposals_count(self.config.data.current_fund.proposals as usize)
                .voting_tokens(
                    token_list
                        .iter()
                        .cloned()
                        .map(|(a, b)| (a, b.into()))
                        .collect(),
                )
                .build()
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        builder = builder.blockchain(blockchain);

        info!("building additional services..");

        if self.config.additional.explorer {
            builder = builder.explorer(ExplorerTemplate {
                connect_to: FOLLOWER.to_string(),
                persistence_mode: PersistenceMode::InMemory,
                address_bech32_prefix: None,
                query_depth_limit: None,
                query_complexity_limit: None,
            });
        }

        if let Some(url) = &self.config.additional.archive {
            builder = builder.archive_conf(discover_archive_input_files(url)?);
        }

        info!("building controllers..");

        let controller = builder.build(self.session_settings.clone())?;

        if !self.skip_qr_generation {
            self.dump_qrs(&controller, &generated_wallet_templates, &root)?;
        }

        info!("dumping vote keys..");

        controller
            .settings()
            .dump_private_vote_keys(ChildPath::new(root));

        info!("building servicing station static data..");

        let parameters = build_servicing_station_parameters(
            &self.config,
            token_list,
            controller.defined_vote_plans(),
            &controller.settings(),
        );
        Ok((controller, parameters))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Qr(#[from] helpers::QrError),
    #[error(transparent)]
    Controller(#[from] crate::mode::standard::VitControllerError),
    #[error("too many vote options provided, only 128 are supported")]
    TooManyOptions,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Archive(#[from] ArchiveConfError),
}
