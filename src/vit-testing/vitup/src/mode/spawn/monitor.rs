use super::NetworkSpawnParams;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Config;
use crate::mode::monitor::MonitorController;
use crate::Result;
use hersir::controller::ProgressBarController;
use hersir::spawn::run_health_check;
use jormungandr_automation::jormungandr::JormungandrRest;
use slave_pool::ThreadPool;
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

    let mut nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        nodes_list.push(monitor_controller.spawn_node(spawn_param)?);
    }

    let vit_station = monitor_controller
        .spawn_vit_station(vit_parameters, template_generator, config.service.version)
        .unwrap();

    let wallet_proxy =
        monitor_controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;

    monitor_controller.monitor_nodes();

    static POOLS: ThreadPool = ThreadPool::new();
    POOLS
        .set_threads(1)
        .expect("could not start statistic thread");

    let rests_and_progress: Vec<(JormungandrRest, ProgressBarController)> = nodes_list
        .iter()
        .map(|x| (x.rest(), x.progress_bar().clone()))
        .collect();

    let wallet_proxy_progress = (wallet_proxy.progress_bar().clone(), wallet_proxy.client());
    let vit_station_progress = (vit_station.progress_bar().clone(), vit_station.rest());

    POOLS.spawn_handle(move || loop {
        run_health_check(&rests_and_progress);

        let (wallet_proxy_progress, wallet_proxy_rest) = &wallet_proxy_progress;

        if let Err(err) = wallet_proxy_rest.health() {
            wallet_proxy_progress.log_info(format!("unhealthy: error({:?})", err));
        } else {
            wallet_proxy_progress.log_info("is up ");
        }

        let (vit_station_progress, vit_station_rest) = &vit_station_progress;

        if let Err(err) = vit_station_rest.health() {
            vit_station_progress.log_info(format!("unhealthy: error({:?})", err));
        } else {
            vit_station_progress.log_info("is up ");
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    });

    ctrlc::set_handler(move || {
        POOLS
            .set_threads(0)
            .expect("could not stop statistic thread");
        for process in nodes_list.iter_mut() {
            process.finish_monitoring();
        }
        vit_station.finish_monitoring();
        wallet_proxy.finish_monitoring();
        tx.send(()).expect("Could not send signal on channel.")
    })
    .expect("Error setting Ctrl-C handler");

    rx.recv().expect("Could not receive from channel.");
    monitor_controller.finalize();
    Ok(())
}
