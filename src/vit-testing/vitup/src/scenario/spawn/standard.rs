use crate::builders::VitBackendSettingsBuilder;
use crate::scenario::spawn::NetworkSpawnParams;
use crate::Result;
use hersir::config::SessionSettings;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    session_settings: SessionSettings,
    network_spawn_params: NetworkSpawnParams,
    quick_setup: VitBackendSettingsBuilder,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let (mut vit_controller, vit_parameters, version) =
        quick_setup.build(session_settings.into())?;

    let mut nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        nodes_list.push(vit_controller.spawn_node(spawn_param)?);
    }
    let _ = vit_controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;
    let _ = vit_controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        network_spawn_params.version(),
    );

    Ok(())
}
