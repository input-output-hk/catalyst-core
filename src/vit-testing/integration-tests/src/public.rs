use chain_impl_mockchain::block::BlockDate;
use jormungandr_lib::interfaces::Explorer;
use jormungandr_scenario_tests::Seed;
use jormungandr_scenario_tests::{
    node::{LeadershipMode, PersistenceMode},
    test::utils,
    Context,
};

use jormungandr_testing_utils::testing::network_builder::SpawnParams;
use jormungandr_testing_utils::testing::node::time;
use jortestkit::prelude::ProgressBarMode;
use std::path::PathBuf;
use tokio;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::scenario::controller::VitController;
use vitup::scenario::settings::VitSettings;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;
const LEADER_1: &str = "Leader1";
const LEADER_2: &str = "Leader2";
const LEADER_3: &str = "Leader3";
const LEADER_4: &str = "Leader4";
const WALLET_NODE: &str = "Wallet_Node";

const DAVID_ADDRESS: &str = "DdzFFzCqrhsktawSMCWJJy3Dpp9BCjYPVecgsMb5U2G7d1ErUUmwSZvfSY3Yjn5njNadfwvebpVNS5cD4acEKSQih2sR76wx2kF4oLXT";
const DAVID_MNEMONICS: &str =
    "tired owner misery large dream glad upset welcome shuffle eagle pulp time";

const EDGAR_ADDRESS: &str = "DdzFFzCqrhsf2sWcZLzXhyLoLZcmw3Zf3UcJ2ozG1EKTwQ6wBY1wMG1tkXtPvEgvE5PKUFmoyzkP8BL4BwLmXuehjRHJtnPj73E5RPMx";
const EDGAR_MNEMONICS: &str =
    "edge club wrap where juice nephew whip entry cover bullet cause jeans";

const FILIP_MNEMONICS: &str =
    "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone";
const FILIP_ADDRESS: &str = "Ae2tdPwUPEZ8og5u4WF5rmSyme5Gvp8RYiLM2u7Vm8CyDQzLN3VYTN895Wk";

#[allow(dead_code)]
pub enum Vote {
    BLANK = 0,
    YES = 1,
    NO = 2,
}
pub fn context() -> Context {
    Context::new(
        Seed::generate(rand::rngs::OsRng),
        PathBuf::from("jormungandr"),
        PathBuf::from("jcli"),
        Some(PathBuf::from("./testing")),
        true,
        ProgressBarMode::Standard,
        "info".to_string(),
    )
}

#[tokio::test]
pub async fn vote_e2e_flow() -> std::result::Result<(), crate::Error> {
    let mut context = context();
    let title = "vote_e2e_flow";
    let scenario_settings = prepare_scenario! {
        title,
        &mut context,
        topology [
            LEADER_1,
            LEADER_2 -> LEADER_1,
            LEADER_3 -> LEADER_1,
            LEADER_4 -> LEADER_1,
            WALLET_NODE -> LEADER_1,LEADER_2,LEADER_3,LEADER_4
        ]
        blockchain {
            consensus = Bft,
            number_of_slots_per_epoch = 60,
            slot_duration = 1,
            leaders = [ LEADER_1, LEADER_2, LEADER_3, LEADER_4 ],
            initials = [
                "account" "Alice" with 500_000_000,
            ],
            committees = [ "Alice" ],
            legacy = [
                "David" address DAVID_ADDRESS mnemonics DAVID_MNEMONICS with 500_000_000,
                "Edgar" address EDGAR_ADDRESS mnemonics EDGAR_MNEMONICS with 500_000_000,
                "Filip" address FILIP_ADDRESS mnemonics FILIP_MNEMONICS with 500_000_000,
            ],
            vote_plans = [
                "fund1" from "Alice" through epochs 0->1->2 as "public" contains proposals = [
                    proposal adds 100 to "rewards" with 3 vote options,
                ]
            ],
        }
    };

    let vit_controller = VitController::new(VitSettings::new(&mut context));
    let mut controller = scenario_settings.build(context).unwrap();

    // bootstrap network
    let leader_1 = controller
        .spawn_node_custom(
            SpawnParams::new(LEADER_1)
                .leader()
                .persistence_mode(PersistenceMode::Persistent)
                .explorer(Explorer { enabled: true }),
        )
        .unwrap();
    leader_1.wait_for_bootstrap().unwrap();
    controller.monitor_nodes();

    //start bft node 2
    let leader_2 = controller
        .spawn_node(
            LEADER_2,
            LeadershipMode::Leader,
            PersistenceMode::Persistent,
        )
        .unwrap();
    leader_2.wait_for_bootstrap().unwrap();

    //start bft node 3
    let leader_3 = controller
        .spawn_node(
            LEADER_3,
            LeadershipMode::Leader,
            PersistenceMode::Persistent,
        )
        .unwrap();
    leader_3.wait_for_bootstrap().unwrap();

    //start bft node 4
    let leader_4 = controller
        .spawn_node(
            LEADER_4,
            LeadershipMode::Leader,
            PersistenceMode::Persistent,
        )
        .unwrap();
    leader_4.wait_for_bootstrap().unwrap();

    // start passive node
    let wallet_node = controller
        .spawn_node_custom(
            SpawnParams::new(WALLET_NODE)
                .passive()
                .persistence_mode(PersistenceMode::Persistent)
                .explorer(Explorer { enabled: true }),
        )
        .unwrap();
    wallet_node.wait_for_bootstrap().unwrap();
    let fund1_vote_plan = controller.vote_plan("fund1").unwrap();

    let blockchain_configuration = controller
        .settings()
        .network_settings
        .block0
        .blockchain_configuration
        .clone();

    let mut vote_plan_parameters_builder = QuickVitBackendSettingsBuilder::new();
    vote_plan_parameters_builder
        .vote_start_epoch(0)
        .tally_start_epoch(1)
        .tally_end_epoch(2)
        .slot_duration_in_seconds(blockchain_configuration.slot_duration.into())
        .slots_in_epoch_count(blockchain_configuration.slots_per_epoch.into())
        .proposals_count(1);

    vote_plan_parameters_builder
        .recalculate_voting_periods_if_needed(blockchain_configuration.block0_date);

    let settings = controller.settings().clone();
    let parameters = vote_plan_parameters_builder.vote_plan_parameters(fund1_vote_plan, settings);
    // start proxy and vit station
    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let vit_station = vit_controller
        .spawn_vit_station(&mut controller, parameters, &mut template_generator)
        .unwrap();
    let wallet_proxy = vit_controller
        .spawn_wallet_proxy(&mut controller, WALLET_NODE)
        .unwrap();

    // wait for spin off
    std::thread::sleep(std::time::Duration::from_secs(1));

    // start mainnet wallets
    let mut david = vit_controller
        .iapyx_wallet(DAVID_MNEMONICS, &wallet_proxy)
        .unwrap();
    david.retrieve_funds().unwrap();
    david.convert_and_send().unwrap();

    let fund1_vote_plan = controller.vote_plan("fund1").unwrap();

    // start voting
    david
        .vote_for(fund1_vote_plan.id(), 0, Vote::YES as u8)
        .unwrap();

    let mut edgar = vit_controller
        .iapyx_wallet(EDGAR_MNEMONICS, &wallet_proxy)
        .unwrap();
    edgar.retrieve_funds().unwrap();
    edgar.convert_and_send().unwrap();

    edgar
        .vote_for(fund1_vote_plan.id(), 0, Vote::YES as u8)
        .unwrap();

    let mut filip = vit_controller
        .iapyx_wallet(FILIP_MNEMONICS, &wallet_proxy)
        .unwrap();
    filip.retrieve_funds().unwrap();
    filip.convert_and_send().unwrap();

    filip
        .vote_for(fund1_vote_plan.id(), 0, Vote::NO as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 5,
    };
    time::wait_for_date(target_date.into(), leader_1.explorer());

    //tally the vote and observe changes
    let rewards_before = leader_1
        .explorer()
        .status()
        .unwrap()
        .data
        .unwrap()
        .status
        .latest_block
        .treasury
        .unwrap()
        .rewards
        .parse::<u64>()
        .unwrap();

    let mut alice = controller.wallet("Alice").unwrap();
    controller
        .fragment_sender()
        .send_public_vote_tally(&mut alice, &fund1_vote_plan.into(), &wallet_node)
        .unwrap();

    time::wait_for_epoch(2, leader_1.explorer());

    let rewards_after = leader_1
        .explorer()
        .status()
        .unwrap()
        .data
        .unwrap()
        .status
        .latest_block
        .treasury
        .unwrap()
        .rewards
        .parse::<u64>()
        .unwrap();

    utils::assert_equals(
        &rewards_before,
        &(rewards_after - 100),
        &format!(
            "{} <> {} rewards were not increased",
            rewards_before, rewards_after
        ),
    )
    .unwrap();

    wallet_node.shutdown().unwrap();
    vit_station.shutdown();
    wallet_proxy.shutdown();
    leader_4.shutdown().unwrap();
    leader_3.shutdown().unwrap();
    leader_2.shutdown().unwrap();
    leader_1.shutdown().unwrap();
    controller.finalize();
    Ok(())
}
