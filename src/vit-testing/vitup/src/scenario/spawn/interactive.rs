use crate::builders::VitBackendSettingsBuilder;
use crate::interactive::VitInteractiveCommandExec;
use crate::interactive::VitUserInteractionController;
use crate::scenario::spawn::NetworkSpawnParams;
use crate::Result;
use hersir::config::SessionSettings;
use hersir::controller::UserInteractionController;
use jortestkit::prelude::UserInteraction;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    session_settings: SessionSettings,
    network_spawn_params: NetworkSpawnParams,
    quick_setup: VitBackendSettingsBuilder,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let (mut vit_controller, vit_parameters, version) =
        quick_setup.build(session_settings.into())?;

    let nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        nodes_list.push(vit_controller.spawn_node(spawn_param)?);
    }
    let wallet_proxy =
        vit_controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;
    let vit_station =
        vit_controller.spawn_vit_station(vit_parameters, template_generator, version)?;

    let user_integration = vit_interaction();
    let mut interaction_controller =
        UserInteractionController::new(vit_controller.hersir_controller().into());
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
