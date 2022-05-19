mod helpers;
mod reviews;
pub mod utils;

use crate::builders::helpers::build_servicing_station_parameters;
use crate::builders::utils::DeploymentTree;
use crate::config::Config;
use crate::mode::standard::{VitController, VitControllerBuilder};
use assert_fs::fixture::ChildPath;
use chain_impl_mockchain::chaintypes::ConsensusVersion;
use chain_impl_mockchain::testing::TestGen;
use chain_impl_mockchain::tokens::identifier::TokenIdentifier;
use chain_impl_mockchain::tokens::minting_policy::MintingPolicy;
use chain_impl_mockchain::value::Value;
pub use helpers::{
    convert_to_blockchain_date, convert_to_human_date, generate_qr_and_hashes,
    VitVotePlanDefBuilder, WalletExtension,
};
use hersir::builder::Blockchain;
use hersir::builder::Node;
use hersir::builder::Topology;
use hersir::builder::WalletTemplate;
use hersir::config::SessionSettings;
pub use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::NumberOfSlotsPerEpoch;
use jormungandr_lib::interfaces::SlotDuration;
pub use reviews::ReviewGenerator;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use thiserror::Error;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;

pub const LEADER_1: &str = "Leader1";
pub const LEADER_2: &str = "Leader2";
pub const LEADER_3: &str = "Leader3";
pub const WALLET_NODE: &str = "Wallet_Node";

#[derive(Clone)]
pub struct VitBackendSettingsBuilder {
    config: Config,
    session_settings: SessionSettings,
    committee_wallet: String,
    //needed for load tests when we rely on secret keys instead of qrs
    skip_qr_generation: bool,
}

impl Default for VitBackendSettingsBuilder {
    fn default() -> Self {
        Self {
            committee_wallet: "committee_1".to_owned(),
            skip_qr_generation: false,
            config: Default::default(),
            session_settings: SessionSettings::default(),
        }
    }
}

impl VitBackendSettingsBuilder {
    pub fn skip_qr_generation(mut self) -> Self {
        self.skip_qr_generation = true;
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
        let passive = Node::new(WALLET_NODE)
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

        let wallets: Vec<(_, _)> = controller
            .defined_wallets()
            .iter()
            .filter(|(_, x)| !x.template().alias().starts_with("committee"))
            .map(|(alias, template)| {
                let wallet: thor::Wallet = ((*template).clone()).into();
                (*alias, wallet)
            })
            .collect();

        generate_qr_and_hashes(wallets, initials, &self.config, &folder).map_err(Into::into)
    }

    pub fn build(self) -> Result<(VitController, ValidVotePlanParameters), Error> {
        let mut builder = VitControllerBuilder::new();

        let vote_blockchain_time = convert_to_blockchain_date(&self.config);

        let mut blockchain = Blockchain::default()
            .with_consensus(ConsensusVersion::Bft)
            .with_slots_per_epoch(
                NumberOfSlotsPerEpoch::new(vote_blockchain_time.slots_per_epoch).unwrap(),
            )
            .with_slot_duration(SlotDuration::new(self.config.blockchain.slot_duration).unwrap());

        println!("building topology..");

        builder = builder.topology(self.build_topology());
        blockchain = blockchain
            .with_leader(LEADER_1)
            .with_leader(LEADER_2)
            .with_leader(LEADER_3);

        println!("building blockchain parameters..");

        blockchain = blockchain
            .with_linear_fee(self.config.blockchain.linear_fees)
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
            blockchain =
                blockchain.with_external_committees(self.config.blockchain.committees.clone());
        }

        let committee_wallet = WalletTemplate::new_account(
            self.committee_wallet.clone(),
            Value(1_000_000_000),
            blockchain.discrimination(),
            Default::default(),
        );
        blockchain = blockchain
            .with_wallet(committee_wallet)
            .with_committee(self.committee_wallet.clone());

        println!("building voting token..");

        let root = self.session_settings.root.path().to_path_buf();
        std::fs::create_dir_all(&root)?;
        let policy = MintingPolicy::new();

        let token_id: TokenIdentifier = TokenIdentifier {
            policy_hash: policy.hash(),
            token_name: TestGen::token_name(),
        };

        let mut file = std::fs::File::create(root.join("voting_token.txt"))?;
        writeln!(file, "{:?}", token_id)?;

        println!("building initials..");

        let mut templates = HashMap::new();
        if self.config.initials.block0.any() {
            blockchain = blockchain.with_external_wallets(
                self.config
                    .initials
                    .block0
                    .external_templates(token_id.clone().into()),
            );
            templates = self.config.initials.block0.templates(
                self.config.data.voting_power,
                blockchain.discrimination(),
                token_id.clone().into(),
            );
            for (wallet, _) in templates.iter().filter(|(x, _)| *x.value() > Value::zero()) {
                blockchain = blockchain.with_wallet(wallet.clone());
            }
        }
        println!("building voteplan..");

        for vote_plan_def in VitVotePlanDefBuilder::default()
            .vote_phases(vote_blockchain_time)
            .options(
                self.config
                    .data
                    .options
                    .0
                    .len()
                    .try_into()
                    .map_err(|_| Error::TooManyOptions)?,
            )
            .split_by(255)
            .fund_name(self.config.data.fund_name.to_string())
            .committee(self.committee_wallet.clone())
            .private(self.config.vote_plan.private)
            .proposals_count(self.config.data.proposals as usize)
            .voting_token(token_id.clone().into())
            .build()
            .into_iter()
        {
            blockchain = blockchain.with_vote_plan(
                vote_plan_def.alias(),
                vote_plan_def.owner(),
                chain_impl_mockchain::certificate::VotePlan::from(vote_plan_def).into(),
            );
        }

        builder = builder.blockchain(blockchain);

        println!("building controllers..");

        let controller = builder.build(self.session_settings.clone())?;

        if !self.skip_qr_generation {
            self.dump_qrs(&controller, &templates, &root)?;
        }

        println!("dumping vote keys..");

        controller
            .settings()
            .dump_private_vote_keys(ChildPath::new(root));

        println!("build servicing station static data..");

        let parameters = build_servicing_station_parameters(
            &self.config,
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
}
