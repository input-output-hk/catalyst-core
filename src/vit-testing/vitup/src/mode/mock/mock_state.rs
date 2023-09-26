use super::{snapshot::VoterSnapshot, Configuration as MockConfig, LedgerState};
use crate::builders::utils::SessionSettingsExtension;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Config;
use crate::mode::mock::NetworkCongestion;
use crate::mode::mock::NetworkCongestionMode;
use crate::mode::standard::VitController;
use chain_impl_mockchain::testing::TestGen;
use hersir::{builder::Wallet as WalletSettings, config::SessionSettings};
use jormungandr_lib::interfaces::{NodeState, NodeStats, NodeStatsDto};
use thiserror::Error;
use thor::WalletAlias;
use tracing::{info, span, Level};
use valgrind::VitVersion;
use vit_servicing_station_lib::db::models::funds::Fund;
use vit_servicing_station_tests::common::data::ArbitrarySnapshotGenerator;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::Snapshot;
use vit_servicing_station_tests::common::data::ValidVotePlanGenerator;

pub struct MockState {
    pub available: bool,
    pub error_code: u16,
    version: VitVersion,
    ledger_state: LedgerState,
    vit_state: Snapshot,
    // ATTENTION: this is not in sync with the ledger
    voters: VoterSnapshot,
    block0_bin: Vec<u8>,
    network_congestion: NetworkCongestion,
    block_account_endpoint_counter: u32,
    controller: VitController,
}

impl MockState {
    pub fn new(params: Config, config: MockConfig) -> Result<Self, Error> {
        let span = span!(Level::INFO, "mock state");
        let _enter = span.enter();

        info!("building default voting event");

        if config.working_dir.exists() {
            std::fs::remove_dir_all(&config.working_dir)?;
        }

        let session_settings = SessionSettings::from_dir(&config.working_dir);
        let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
        let (controller, vit_parameters) = VitBackendSettingsBuilder::default()
            .config(&params)
            .session_settings(session_settings)
            .span(span!(Level::INFO, "voting data builder"))
            .build()?;

        info!("building default voting event done");

        let mut generator = ValidVotePlanGenerator::new(vit_parameters);
        let mut vit_state = generator.build(&mut template_generator);
        vit_state
            .funds_mut()
            .extend(ArbitrarySnapshotGenerator::default().funds());

        let reviews = vit_state.advisor_reviews();

        //perform db view operations
        for proposal in vit_state.proposals_mut().iter_mut() {
            proposal.proposal.reviews_count = reviews
                .iter()
                .filter(|review| review.proposal_id.to_string() == proposal.proposal.proposal_id)
                .count() as i32;
        }

        Ok(Self {
            available: true,
            error_code: 400,
            ledger_state: LedgerState::new(controller.settings().block0)?,
            network_congestion: NetworkCongestion::new(&vit_state),
            vit_state,
            version: VitVersion {
                service_version: params.service.version,
            },
            voters: VoterSnapshot::from_config_or_default(&params.initials.snapshot)?,
            block0_bin: jortestkit::file::get_file_as_byte_vec(controller.block0_file())?,
            block_account_endpoint_counter: 0,
            controller,
        })
    }

    pub fn set_block_account_endpoint(&mut self, block_account_endpoint_counter: u32) {
        self.block_account_endpoint_counter = block_account_endpoint_counter;
    }

    pub fn decrement_block_account_endpoint(&mut self) {
        if self.block_account_endpoint_counter > 0 {
            self.block_account_endpoint_counter -= 1;
        }
    }

    pub fn defined_wallets(&self) -> Vec<(WalletAlias, &WalletSettings)> {
        self.controller.defined_wallets()
    }

    pub fn reset_block_account_endpoint(&mut self) {
        self.block_account_endpoint_counter = 0;
    }

    pub fn block_account_endpoint(&self) -> u32 {
        self.block_account_endpoint_counter
    }

    pub fn version(&self) -> VitVersion {
        VitVersion {
            service_version: self.version.service_version.clone(),
        }
    }

    pub fn block0_bin(&self) -> Vec<u8> {
        self.block0_bin.clone()
    }

    pub fn set_congestion(&mut self, network_congestion_mode: NetworkCongestionMode) {
        self.network_congestion.set_mode(network_congestion_mode);
    }

    pub fn set_version(&mut self, version: String) {
        self.version = VitVersion {
            service_version: version,
        }
    }

    pub fn vit(&self) -> &Snapshot {
        &self.vit_state
    }

    pub fn vit_mut(&mut self) -> &mut Snapshot {
        &mut self.vit_state
    }

    pub fn voters(&self) -> &VoterSnapshot {
        &self.voters
    }

    pub fn voters_mut(&mut self) -> &mut VoterSnapshot {
        &mut self.voters
    }

    pub fn ledger(&self) -> &LedgerState {
        &self.ledger_state
    }

    pub fn ledger_mut(&mut self) -> &mut LedgerState {
        &mut self.ledger_state
    }

    pub fn set_fund_id(&mut self, id: i32) {
        let funds = self.vit_state.funds_mut();
        let fund = funds.first_mut().unwrap();

        fund.id = id;

        for challenge in fund.challenges.iter_mut() {
            challenge.fund_id = id;
        }

        for vote_plan in fund.chain_vote_plans.iter_mut() {
            vote_plan.fund_id = id;
        }

        for challenge in self.vit_state.challenges_mut() {
            challenge.fund_id = id;
        }

        for proposal in self.vit_state.proposals_mut() {
            proposal.proposal.fund_id = id;
        }
    }

    pub fn update_fund(&mut self, new_fund: Fund) {
        let old = self
            .vit_state
            .funds()
            .iter()
            .position(|f| f.id == new_fund.id);
        self.vit_state.funds_mut().push(new_fund);
        if let Some(pos) = old {
            self.vit_state.funds_mut().swap_remove(pos);
        }
    }

    pub fn node_stats(&self) -> NodeStatsDto {
        let settings = self.ledger().settings();

        let uptime = std::time::SystemTime::now()
            .duration_since(settings.block0_time.into())
            .unwrap()
            .as_secs();

        let network_congestion_data = self.network_congestion.calculate(self);

        NodeStatsDto {
            version: "jormungandr 0.13.0".to_string(),
            state: NodeState::Running,
            stats: Some(NodeStats {
                block_recv_cnt: uptime / 3,
                last_block_content_size: 0,
                last_block_date: Some(self.ledger_state.current_blockchain_age().to_string()),
                last_block_fees: 0,
                last_block_hash: Some(TestGen::hash().to_string()),
                last_block_height: Some((uptime / settings.slot_duration).to_string()),
                last_block_sum: 0,
                last_block_time: Some(self.ledger_state.curr_slot_start_time()),
                last_block_tx: 0,
                last_received_block_time: None,
                block_content_size_avg: network_congestion_data.block_content_size_avg,
                peer_available_cnt: 2,
                peer_connected_cnt: 2,
                peer_quarantined_cnt: 0,
                peer_total_cnt: 2,
                tx_recv_cnt: network_congestion_data.received_fragments_count as u64,
                mempool_usage_ratio: network_congestion_data.mempool_usage_ratio,
                mempool_tx_count: network_congestion_data.mempool_tx_count,
                tx_rejected_cnt: network_congestion_data.rejected_fragments_count as u64,
                votes_cast: network_congestion_data.received_fragments_count as u64,
                uptime: Some(uptime),
            }),
        }
    }
}

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    #[error(transparent)]
    Builder(#[from] crate::builders::Error),
    #[error(transparent)]
    Ledger(#[from] super::ledger_state::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Snapshot(#[from] mainnet_tools::snapshot::Error),
}
