use super::NetworkSpawnParams;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Config;
use crate::mode::monitor::MonitorController;
use crate::Result;
use std::sync::mpsc::channel;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    network_spawn_params: NetworkSpawnParams,
    config: Config,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let (tx, rx): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = channel();

    let (vit_controller, vit_parameters) = VitBackendSettingsBuilder::default()
        .config(&config)
        .session_settings(network_spawn_params.session_settings())
        .build()?;

    let hersir_monitor_controller = hersir::controller::MonitorController::new(
        vit_controller.hersir_controller(),
        network_spawn_params.session_settings(),
    )?;
    let mut monitor_controller = MonitorController::new(vit_controller, hersir_monitor_controller);

    println!("Waiting for Ctrl-C to exit..");
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    monitor_controller.monitor_nodes();

    let mut nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        nodes_list.push(monitor_controller.spawn_node(spawn_param)?);
    }

    let _vit_station = monitor_controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        config.service.version,
    )?;
    let _wallet_proxy =
        monitor_controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;

    rx.recv().expect("Could not receive from channel.");
    monitor_controller.finalize();

    Ok(())
}
