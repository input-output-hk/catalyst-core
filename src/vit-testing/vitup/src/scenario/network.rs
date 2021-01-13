use crate::interactive::VitInteractiveCommandExec;
use crate::interactive::VitUserInteractionController;
use crate::setup::{
    QuickVitBackendSettingsBuilder, LEADER_1, LEADER_2, LEADER_3, LEADER_4, WALLET_NODE,
};
use crate::wallet::WalletProxySpawnParams;
use crate::Result;
use jormungandr_lib::interfaces::Explorer;
use jormungandr_scenario_tests::interactive::UserInteractionController;
use jormungandr_scenario_tests::{
    node::{LeadershipMode, PersistenceMode},
    scenario::{repository::ScenarioResult, Context},
};
use jormungandr_testing_utils::testing::network_builder::SpawnParams;
use jortestkit::prelude::UserInteraction;
use rand_chacha::ChaChaRng;

#[allow(unreachable_code)]
#[allow(clippy::empty_loop)]
pub fn vote_backend(
    context: Context<ChaChaRng>,
    mut quick_setup: QuickVitBackendSettingsBuilder,
    interactive: bool,
    endpoint: String,
) -> Result<ScenarioResult> {
    let (vit_controller, mut controller, vit_parameters) = quick_setup.build(context)?;

    // bootstrap network
    let leader_1 = controller.spawn_node_custom(
        SpawnParams::new(LEADER_1)
            .leader()
            .persistence_mode(PersistenceMode::Persistent)
            .explorer(Explorer { enabled: true }),
    )?;
    leader_1.wait_for_bootstrap()?;
    controller.monitor_nodes();

    //start bft node 2
    let leader_2 = controller.spawn_node(
        LEADER_2,
        LeadershipMode::Leader,
        PersistenceMode::Persistent,
    )?;
    leader_2.wait_for_bootstrap()?;

    //start bft node 3
    let leader_3 = controller.spawn_node(
        LEADER_3,
        LeadershipMode::Leader,
        PersistenceMode::Persistent,
    )?;
    leader_3.wait_for_bootstrap()?;

    //start bft node 4
    let leader_4 = controller.spawn_node(
        LEADER_4,
        LeadershipMode::Leader,
        PersistenceMode::Persistent,
    )?;
    leader_4.wait_for_bootstrap()?;

    // start passive node
    let wallet_node = controller.spawn_node_custom(
        SpawnParams::new(WALLET_NODE)
            .passive()
            .persistence_mode(PersistenceMode::Persistent)
            .explorer(Explorer { enabled: true }),
    )?;
    wallet_node.wait_for_bootstrap()?;

    // start proxy and vit station
    let vit_station = vit_controller.spawn_vit_station(&mut controller, vit_parameters)?;
    let wallet_proxy = vit_controller.spawn_wallet_proxy_custom(
        &mut controller,
        WalletProxySpawnParams::new(WALLET_NODE).with_base_address(endpoint),
    )?;

    match interactive {
        true => {
            let user_integration = vit_interaction();
            let mut interaction_controller = UserInteractionController::new(controller);
            let mut vit_interaction_controller: VitUserInteractionController = Default::default();
            let nodes = interaction_controller.nodes_mut();
            nodes.push(leader_1);
            nodes.push(leader_2);
            nodes.push(leader_3);
            nodes.push(leader_4);
            nodes.push(wallet_node);
            vit_interaction_controller.proxies_mut().push(wallet_proxy);
            vit_interaction_controller
                .vit_stations_mut()
                .push(vit_station);

            let mut command_exec = VitInteractiveCommandExec {
                controller: interaction_controller,
                vit_controller: vit_interaction_controller,
            };

            user_integration.interact(&mut command_exec)?;
            command_exec.tear_down();
        }
        false => loop {},
    }

    Ok(ScenarioResult::passed(""))
}

fn vit_interaction() -> UserInteraction {
    UserInteraction::new(
        "jormungandr-scenario-tests".to_string(),
        "jormungandr vit backend".to_string(),
        "type command:".to_string(),
        "exit".to_string(),
        ">".to_string(),
        vec![
            "You can control each aspect of backend:".to_string(),
            "- spawn nodes,".to_string(),
            "- send fragments,".to_string(),
            "- filter logs,".to_string(),
            "- show node stats and data.".to_string(),
        ],
    )
}
