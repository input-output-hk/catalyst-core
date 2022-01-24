use crate::builders::VitBackendSettingsBuilder;
use crate::scenario::monitor::MonitorController;
use crate::scenario::spawn::NetworkSpawnParams;
use crate::Result;
use hersir::config::SessionSettings;
use std::sync::mpsc::channel;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    session_settings: SessionSettings,
    network_spawn_params: NetworkSpawnParams,
    quick_setup: VitBackendSettingsBuilder,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let (tx, rx): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = channel();

    let (mut vit_controller, vit_parameters, version) =
        quick_setup.build(session_settings.into())?;
    let hersir_monitor_controller = hersir::controller::MonitorController::new(
        vit_controller.hersir_controller(),
        session_settings.clone().into(),
    )?;
    let monitor_controller = MonitorController::new(vit_controller, hersir_monitor_controller);

    println!("Waiting for Ctrl-C to exit..");
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    monitor_controller.monitor_nodes();

    let nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        monitor_controller.spawn_node(spawn_param)?;
    }

    let vit_station = monitor_controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        network_spawn_params.version(),
    )?;
    let wallet_proxy =
        monitor_controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;

    rx.recv().expect("Could not receive from channel.");
    for node in nodes_list.iter_mut() {
        node.shutdown().unwrap();
    }
    vit_station.shutdown();
    wallet_proxy.shutdown();
    monitor_controller.finalize();

    Ok(())
}
