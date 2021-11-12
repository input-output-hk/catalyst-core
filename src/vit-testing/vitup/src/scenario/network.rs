use crate::builders::{VitBackendSettingsBuilder, LEADER_1, LEADER_2, LEADER_3, WALLET_NODE};
use crate::interactive::VitInteractiveCommandExec;
use crate::interactive::VitUserInteractionController;
use crate::manager::State;
use crate::manager::{ControlContext, ControlContextLock, ManagerService};
use crate::scenario::controller::VitController;
use crate::vit_station::VitStationController;
use crate::wallet::WalletProxyController;
use crate::wallet::WalletProxySpawnParams;
use crate::Result;
use jormungandr_scenario_tests::interactive::UserInteractionController;
use jormungandr_scenario_tests::node::Node as NodeController;
use jormungandr_scenario_tests::scenario::Controller;
use jormungandr_scenario_tests::{
    node::{LeadershipMode, PersistenceMode},
    scenario::Context,
};
use jormungandr_testing_utils::testing::network::SpawnParams;
use jortestkit::prelude::UserInteraction;
use rand_chacha::ChaChaRng;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use valgrind::Protocol;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn setup_network(
    controller: &mut Controller,
    vit_controller: &mut VitController,
    vit_parameters: ValidVotePlanParameters,
    vit_data_generator: &mut dyn ValidVotingTemplateGenerator,
    endpoint: String,
    protocol: &Protocol,
    vit_version: String,
) -> Result<(
    Vec<NodeController>,
    VitStationController,
    WalletProxyController,
)> {
    println!("Spawning leader 1..");

    // bootstrap network
    let leader_1 = controller.spawn_node_custom(
        SpawnParams::new(LEADER_1)
            .leader()
            .persistence_mode(PersistenceMode::Persistent),
    )?;
    leader_1.wait_for_bootstrap()?;
    controller.monitor_nodes();

    println!("Spawning leader 2..");

    //start bft node 2
    let leader_2 = controller.spawn_node(
        LEADER_2,
        LeadershipMode::Leader,
        PersistenceMode::Persistent,
    )?;
    leader_2.wait_for_bootstrap()?;

    println!("Spawning leader 3..");

    //start bft node 3
    let leader_3 = controller.spawn_node(
        LEADER_3,
        LeadershipMode::Leader,
        PersistenceMode::Persistent,
    )?;
    leader_3.wait_for_bootstrap()?;

    println!("Spawning wallet node..");

    // start passive node
    let wallet_node = controller.spawn_node_custom(
        SpawnParams::new(WALLET_NODE)
            .passive()
            .persistence_mode(PersistenceMode::Persistent)
            .persistent_fragment_log(controller.working_directory().path().join("persistent_log")),
    )?;
    wallet_node.wait_for_bootstrap()?;

    println!("Spawning vit station..");

    // start proxy and vit station
    let vit_station = vit_controller.spawn_vit_station(
        controller,
        vit_parameters,
        vit_data_generator,
        vit_version,
    )?;
    let wallet_proxy = vit_controller.spawn_wallet_proxy_custom(
        controller,
        WalletProxySpawnParams::new(WALLET_NODE)
            .with_base_address(endpoint)
            .with_protocol(protocol.clone()),
    )?;

    println!("Backend network is up");

    Ok((
        vec![leader_1, leader_2, leader_3, wallet_node],
        vit_station,
        wallet_proxy,
    ))
}

pub fn interactive_mode(
    controller: Controller,
    nodes_list: Vec<NodeController>,
    vit_station: VitStationController,
    wallet_proxy: WalletProxyController,
) -> Result<()> {
    let user_integration = vit_interaction();
    let mut interaction_controller = UserInteractionController::new(controller);
    let mut vit_interaction_controller: VitUserInteractionController = Default::default();
    let nodes = interaction_controller.nodes_mut();
    nodes.extend(nodes_list);
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
    Ok(())
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

#[allow(clippy::empty_loop)]
#[allow(unreachable_code)]
pub fn endless_mode() -> Result<()> {
    loop {}
    Ok(())
}

#[allow(unreachable_code)]
pub fn service_mode<P: AsRef<Path> + Clone>(
    context: Context<ChaChaRng>,
    working_dir: P,
    mut quick_setup: VitBackendSettingsBuilder,
    endpoint: String,
    token: Option<String>,
) -> Result<()> {
    let protocol = quick_setup.protocol().clone();

    let control_context = Arc::new(Mutex::new(ControlContext::new(
        working_dir.clone(),
        quick_setup.parameters().clone(),
        token,
    )));

    let mut manager = ManagerService::new(control_context.clone());
    manager.spawn();

    loop {
        if manager.request_to_start() {
            let testing_directory = working_dir.as_ref();
            if testing_directory.exists() {
                std::fs::remove_dir_all(testing_directory)?;
            }

            let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

            let parameters = manager.setup();
            quick_setup.upload_parameters(parameters);
            manager.clear_requests();
            single_run(
                control_context.clone(),
                context.clone(),
                quick_setup.clone(),
                endpoint.clone(),
                &protocol,
                &mut template_generator,
            )?;
        }

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
    Ok(())
}

pub fn single_run(
    control_context: ControlContextLock,
    context: Context<ChaChaRng>,
    mut quick_setup: VitBackendSettingsBuilder,
    endpoint: String,
    protocol: &Protocol,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    {
        let mut control_context = control_context.lock().unwrap();
        let state = control_context.state_mut();
        *state = State::Starting;
    }

    let (mut vit_controller, mut controller, vit_parameters, version) =
        quick_setup.build(context)?;
    let (mut nodes_list, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        template_generator,
        endpoint,
        protocol,
        version,
    )?;

    {
        let mut control_context = control_context.lock().unwrap();
        let state = control_context.state_mut();
        *state = State::Running;
    }

    loop {
        if control_context.lock().unwrap().request_to_stop() {
            {
                let mut control_context = control_context.lock().unwrap();
                let state = control_context.state_mut();
                *state = State::Stopping;
            }

            vit_station.shutdown();
            wallet_proxy.shutdown();
            for node in nodes_list.iter_mut() {
                node.shutdown()?;
            }

            controller.finalize();
            {
                let mut control_context = control_context.lock().unwrap();
                let state = control_context.state_mut();
                *state = State::Idle;
            }
            return Ok(());
        }

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}
