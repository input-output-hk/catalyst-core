use super::NetworkSpawnParams;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Config;
use crate::Result;
use std::sync::mpsc::channel;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    network_spawn_params: NetworkSpawnParams,
    config: Config,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let (tx, rx): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = channel();

    let (mut vit_controller, vit_parameters) = VitBackendSettingsBuilder::default()
        .config(&config)
        .session_settings(network_spawn_params.session_settings())
        .build()?;

    let mut nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        nodes_list.push(vit_controller.spawn_node(spawn_param)?);
    }
    let _wallet_proxy =
        vit_controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;

    let mut vit_stations = vec![];

    vit_stations.push(vit_controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        network_spawn_params.version(),
    ));

    if config.additional.archive.is_some() {
        vit_stations.push(vit_controller.spawn_vit_station_archive(network_spawn_params.version()));
    };

    let mut explorers = vec![];

    if config.additional.explorer {
        explorers.push(vit_controller.spawn_explorer());
    };

    println!("Waiting for Ctrl-C to exit..");
    ctrlc::set_handler(move || {
        println!("Shutting down..");
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    rx.recv().expect("Could not receive from channel.");
    println!("Exited");
    #[allow(unreachable_code)]
    Ok(())
}
