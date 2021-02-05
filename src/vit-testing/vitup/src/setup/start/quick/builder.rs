use crate::config::VitStartParameters;
use crate::scenario::controller::VitController;
use crate::scenario::controller::VitControllerBuilder;
use crate::{config::Initials, Result};
use assert_fs::fixture::{ChildPath, PathChild};
use chain_crypto::SecretKey;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_impl_mockchain::vote::PayloadType;
use chain_impl_mockchain::{
    testing::scenario::template::{ProposalDefBuilder, VotePlanDefBuilder},
    value::Value,
};
use jormungandr_testing_utils::wallet::LinearFee;
use chain_vote::committee::ElectionPublicKey;
use chrono::naive::NaiveDateTime;
use iapyx::Protocol;
use jormungandr_lib::time::SecondsSinceUnixEpoch;
use jormungandr_scenario_tests::scenario::settings::Settings;
use jormungandr_scenario_tests::scenario::{
    ActiveSlotCoefficient, ConsensusVersion, ContextChaCha, Controller, KESUpdateSpeed, Milli,
    NumberOfSlotsPerEpoch, SlotDuration, Topology, TopologyBuilder,
};
use jormungandr_testing_utils::testing::network_builder::{
    Blockchain, Node, WalletTemplate, WalletType,
};
use jormungandr_testing_utils::{qr_code::KeyQrCode, wallet::ElectionPublicKeyExtension};
use std::collections::HashMap;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;

pub const LEADER_1: &str = "Leader1";
pub const LEADER_2: &str = "Leader2";
pub const LEADER_3: &str = "Leader3";
pub const LEADER_4: &str = "Leader4";
pub const WALLET_NODE: &str = "Wallet_Node";

#[derive(Clone)]
pub struct QuickVitBackendSettingsBuilder {
    parameters: VitStartParameters,
    committe_wallet_name: String,
    title: String,
}

impl Default for QuickVitBackendSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl QuickVitBackendSettingsBuilder {
    pub fn new() -> Self {
        Self {
            parameters: Default::default(),
            title: "vit_backend".to_owned(),
            committe_wallet_name: "committee".to_owned(),
        }
    }

    pub fn parameters(&self) -> &VitStartParameters {
        &self.parameters
    }

    pub fn with_protocol(&mut self, protocol: Protocol) -> &mut Self {
        self.parameters.protocol = protocol;
        self
    }

    pub fn protocol(&self) -> &Protocol {
        &self.parameters.protocol
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn initials(&mut self, initials: Initials) -> &mut Self {
        self.parameters.initials = initials;
        self
    }

    pub fn initials_count(&mut self, initials_count: usize, pin: &str) -> &mut Self {
        self.initials(Initials::new_above_threshold(
            initials_count,
            &pin.to_string(),
        ));
        self
    }

    pub fn vote_start_epoch(&mut self, vote_start_epoch: u32) -> &mut Self {
        self.parameters.vote_start = vote_start_epoch as u64;
        self
    }

    pub fn tally_start_epoch(&mut self, tally_start_epoch: u32) -> &mut Self {
        self.parameters.vote_tally = tally_start_epoch as u64;
        self
    }
    pub fn tally_end_epoch(&mut self, tally_end_epoch: u32) -> &mut Self {
        self.parameters.tally_end = tally_end_epoch as u64;
        self
    }

    pub fn slot_duration_in_seconds(&mut self, slot_duration: u8) -> &mut Self {
        self.parameters.slot_duration = slot_duration;
        self
    }
    pub fn slots_in_epoch_count(&mut self, slots_in_epoch: u32) -> &mut Self {
        self.parameters.slots_per_epoch = slots_in_epoch;
        self
    }
    pub fn proposals_count(&mut self, proposals_count: u32) -> &mut Self {
        self.parameters.proposals = proposals_count;
        self
    }
    pub fn voting_power(&mut self, voting_power: u64) -> &mut Self {
        self.parameters.voting_power = voting_power;
        self
    }

    pub fn next_vote_timestamp(&mut self, next_vote_timestamp: Option<String>) -> &mut Self {
        if let Some(timestamp) = next_vote_timestamp {
            self.parameters.next_vote_start_time =
                Some(NaiveDateTime::parse_from_str(&timestamp, FORMAT).unwrap());
        }
        self
    }

    pub fn vote_start_timestamp(&mut self, vote_start_timestamp: Option<String>) -> &mut Self {
        if let Some(timestamp) = vote_start_timestamp {
            self.parameters.vote_start_timestamp =
                Some(NaiveDateTime::parse_from_str(&timestamp, FORMAT).unwrap());
        }
        self
    }

    pub fn tally_start_timestamp(&mut self, tally_start_timestamp: Option<String>) -> &mut Self {
        if let Some(timestamp) = tally_start_timestamp {
            self.parameters.tally_start_timestamp =
                Some(NaiveDateTime::parse_from_str(&timestamp, FORMAT).unwrap());
        }
        self
    }

    pub fn tally_end_timestamp(&mut self, tally_end_timestamp: Option<String>) -> &mut Self {
        if let Some(timestamp) = tally_end_timestamp {
            self.parameters.tally_end_timestamp =
                Some(NaiveDateTime::parse_from_str(&timestamp, FORMAT).unwrap());
        }
        self
    }

    pub fn fund_name(&self) -> String {
        self.parameters.fund_name.to_string()
    }

    pub fn private(&mut self, private: bool) {
        self.parameters.private = private;
    }

    pub fn recalculate_voting_periods_if_needed(&mut self, block0_date: SecondsSinceUnixEpoch) {
        let epoch_duration: u64 =
            self.parameters.slot_duration as u64 * self.parameters.slots_per_epoch as u64;
        if self.parameters.vote_start_timestamp.is_none() {
            println!(
                "Current date {:?}",
                NaiveDateTime::from_timestamp(block0_date.to_secs() as i64, 0)
            );
            let vote_start_timestamp =
                block0_date.to_secs() + epoch_duration * self.parameters.vote_start;
            self.parameters.vote_start_timestamp = Some(NaiveDateTime::from_timestamp(
                vote_start_timestamp as i64,
                0,
            ));
            let tally_start_timestamp =
                block0_date.to_secs() + epoch_duration * self.parameters.vote_tally;
            self.parameters.tally_start_timestamp = Some(NaiveDateTime::from_timestamp(
                tally_start_timestamp as i64,
                0,
            ));
            let tally_end_timestamp =
                block0_date.to_secs() + epoch_duration * self.parameters.tally_end;
            self.parameters.tally_end_timestamp =
                Some(NaiveDateTime::from_timestamp(tally_end_timestamp as i64, 0));
        }

        if self.parameters.next_vote_start_time.is_none() {
            let timestamp = SecondsSinceUnixEpoch::now().to_secs()
                + epoch_duration * self.parameters.tally_end
                + 10_000;
            self.parameters.next_vote_start_time =
                Some(NaiveDateTime::from_timestamp(timestamp as i64, 0));
        }
    }

    pub fn upload_parameters(&mut self, parameters: VitStartParameters) {
        self.parameters = parameters;
    }

    pub fn vote_plan_parameters(
        &self,
        vote_plan: VotePlanDef,
        settings: &Settings,
    ) -> ValidVotePlanParameters {
        let mut parameters = ValidVotePlanParameters::new(vote_plan);
        parameters.set_voting_power_threshold((self.parameters.voting_power * 1_000_000) as i64);
        parameters.set_voting_start(self.parameters.vote_start_timestamp.unwrap().timestamp());
        parameters
            .set_voting_tally_start(self.parameters.tally_start_timestamp.unwrap().timestamp());
        parameters.set_voting_tally_end(self.parameters.tally_end_timestamp.unwrap().timestamp());
        parameters
            .set_next_fund_start_time(self.parameters.next_vote_start_time.unwrap().timestamp());
        parameters.set_fund_id(self.parameters.fund_id);

        if self.parameters.private {
            let mut committee_wallet = settings
                .network_settings
                .wallets
                .get(&self.committe_wallet_name)
                .unwrap()
                .clone();
            let identifier = committee_wallet.identifier();
            let private_key_data = settings
                .private_vote_plans
                .values()
                .next()
                .unwrap()
                .get(&identifier.into())
                .unwrap();
            let key: ElectionPublicKey = private_key_data.encrypting_vote_key();
            parameters.set_vote_encryption_key(key.to_base32().unwrap());
        }
        parameters
    }

    pub fn build_topology(&mut self) -> Topology {
        let mut topology_builder = TopologyBuilder::new();

        // Leader 1
        let leader_1 = Node::new(LEADER_1);
        topology_builder.register_node(leader_1);

        // leader 2
        let mut leader_2 = Node::new(LEADER_2);
        leader_2.add_trusted_peer(LEADER_1);
        topology_builder.register_node(leader_2);

        // leader 3
        let mut leader_3 = Node::new(LEADER_3);
        leader_3.add_trusted_peer(LEADER_1);
        leader_3.add_trusted_peer(LEADER_2);
        topology_builder.register_node(leader_3);

        // leader 4
        let mut leader_4 = Node::new(LEADER_4);
        leader_4.add_trusted_peer(LEADER_1);
        leader_4.add_trusted_peer(LEADER_2);
        leader_4.add_trusted_peer(LEADER_3);
        topology_builder.register_node(leader_4);

        // passive
        let mut passive = Node::new(WALLET_NODE);
        passive.add_trusted_peer(LEADER_1);
        passive.add_trusted_peer(LEADER_2);
        passive.add_trusted_peer(LEADER_3);
        passive.add_trusted_peer(LEADER_4);

        topology_builder.register_node(passive);

        topology_builder.build()
    }

    pub fn build_vote_plan(&mut self) -> VotePlanDef {
        let mut vote_plan_builder = VotePlanDefBuilder::new(&self.fund_name());
        vote_plan_builder.owner(&self.committe_wallet_name);

        if self.parameters.private {
            vote_plan_builder.payload_type(PayloadType::Private);
        }
        vote_plan_builder.vote_phases(
            self.parameters.vote_start as u32,
            self.parameters.vote_tally as u32,
            self.parameters.tally_end as u32,
        );

        for _ in 0..self.parameters.proposals {
            let mut proposal_builder = ProposalDefBuilder::new(
                chain_impl_mockchain::testing::VoteTestGen::external_proposal_id(),
            );
            proposal_builder.options(3);

            proposal_builder.action_off_chain();
            vote_plan_builder.with_proposal(&mut proposal_builder);
        }

        vote_plan_builder.build()
    }
    pub fn dump_qrs(
        &self,
        controller: &Controller,
        initials: &HashMap<WalletTemplate, String>,
        child: &ChildPath,
    ) -> Result<()> {
        let folder = child.child("qr-codes");
        std::fs::create_dir_all(folder.path())?;

        for (alias, _template) in controller
            .wallets()
            .filter(|(_, x)| *x.template().wallet_type() == WalletType::UTxO)
        {
            let wallet = controller.wallet(alias)?;

            let pin = initials
                .iter()
                .find_map(|(template, pin)| {
                    if template.alias() == alias {
                        Some(pin)
                    } else {
                        None
                    }
                })
                .unwrap();
            let png = folder.child(format!("{}_{}.png", alias, pin));
            wallet.save_qr_code(png.path(), &pin_to_bytes(&pin));
        }

        let zero_funds_initial_counts = self.parameters.initials.zero_funds_count();

        if zero_funds_initial_counts > 0 {
            let zero_funds_pin = self.parameters.initials.zero_funds_pin().unwrap();

            for i in 1..zero_funds_initial_counts + 1 {
                let sk = SecretKey::generate(rand::thread_rng());
                let qr = KeyQrCode::generate(sk.clone(), &pin_to_bytes(&zero_funds_pin));
                let img = qr.to_img();
                let png = folder.child(format!("zero_funds_{}_{}.png", i, zero_funds_pin));
                img.save(png.path())?;
            }
        }
        Ok(())
    }

    pub fn build(
        &mut self,
        mut context: ContextChaCha,
    ) -> Result<(VitController, Controller, ValidVotePlanParameters)> {
        let mut builder = VitControllerBuilder::new(&self.title);

        builder.set_topology(self.build_topology());

        let mut blockchain = Blockchain::new(
            ConsensusVersion::Bft,
            NumberOfSlotsPerEpoch::new(self.parameters.slots_per_epoch)
                .expect("valid number of slots per epoch"),
            SlotDuration::new(self.parameters.slot_duration)
                .expect("valid slot duration in seconds"),
            KESUpdateSpeed::new(46800).expect("valid kes update speed in seconds"),
            ActiveSlotCoefficient::new(Milli::from_millis(700))
                .expect("active slot coefficient in millis"),
        );

        blockchain.add_leader(LEADER_1);
        blockchain.add_leader(LEADER_2);
        blockchain.add_leader(LEADER_3);
        blockchain.add_leader(LEADER_4);
        blockchain.set_linear_fee(LinearFee::new(0,0,0,));

        let committe_wallet =
            WalletTemplate::new_account(&self.committe_wallet_name, Value(1_000_000));
        blockchain.add_wallet(committe_wallet);

        let child = context.child_directory(self.title());

        let initials = self
            .parameters
            .initials
            .templates(self.parameters.voting_power);
        for (wallet, _) in initials.iter().filter(|(x, _)| *x.value() > Value::zero()) {
            blockchain.add_wallet(wallet.clone());
        }

        blockchain.add_committee(&self.committe_wallet_name);

        let mut vote_plan_def = self.build_vote_plan();
        blockchain.add_vote_plan(vote_plan_def.clone());
        builder.set_blockchain(blockchain);
        builder.build_settings(&mut context);

        let (vit_controller, controller) = builder.build_controllers(context)?;

        self.dump_qrs(&controller, &initials, &child)?;

        controller.settings().dump_private_vote_keys(child);

        self.recalculate_voting_periods_if_needed(
            controller
                .settings()
                .network_settings
                .block0
                .blockchain_configuration
                .block0_date,
        );

        if self.parameters.private {
            let vote_plan = controller
                .settings()
                .private_vote_plans
                .values()
                .next()
                .unwrap();
            let committee_keys = vote_plan_def.committee_keys_mut();

            for key in vote_plan.member_public_keys().iter() {
                committee_keys.push(key.clone());
            }
        }
        let parameters = self.vote_plan_parameters(vote_plan_def, &controller.settings());

        Ok((vit_controller, controller, parameters))
    }
}

pub fn pin_to_bytes(pin: &str) -> Vec<u8> {
    pin.chars().map(|x| x.to_digit(10).unwrap() as u8).collect()
}
