use super::NetworkSpawnParams;
use crate::builders::VitBackendSettingsBuilder;
use crate::Result;
use std::sync::mpsc::channel;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    network_spawn_params: NetworkSpawnParams,
    mut quick_setup: VitBackendSettingsBuilder,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let (tx, rx): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = channel();

    let (mut vit_controller, vit_parameters, _) =
        quick_setup.build(network_spawn_params.session_settings())?;

    let mut nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        nodes_list.push(vit_controller.spawn_node(spawn_param)?);
    }
    let _wallet_proxy =
        vit_controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;
    let _vit_station = vit_controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        network_spawn_params.version(),
    );

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
