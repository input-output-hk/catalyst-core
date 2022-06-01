use super::NetworkSpawnParams;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Config;
use crate::mode::monitor::MonitorController;
use crate::mode::service::manager::{ControlContext, ControlContextLock, ManagerService, State};
use crate::mode::standard::ValidVotingTemplateGenerator;
use crate::Result;
use std::sync::{Arc, Mutex};
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;

pub fn spawn_network(
    network_params: NetworkSpawnParams,
    config: Config,
    _template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let working_dir = network_params.session_settings().root;
    let control_context = Arc::new(Mutex::new(ControlContext::new(
        working_dir.path(),
        config,
        network_params.token(),
    )));

    let mut manager = ManagerService::new(control_context.clone());
    manager.spawn();

    loop {
        //TODO: remove this dirty fix and implement CLONE for ValidVotingTemplateGenerator
        let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

        if manager.request_to_start() {
            if working_dir.path().exists() {
                std::fs::remove_dir_all(working_dir.path())?;
            }

            manager.clear_requests();
            single_run(
                control_context.clone(),
                network_params.clone(),
                &mut template_generator,
            )?;
        }

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

pub fn single_run(
    control_context: ControlContextLock,
    network_params: NetworkSpawnParams,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    {
        let mut control_context = control_context.lock().unwrap();
        let state = control_context.state_mut();
        *state = State::Starting;
    }

    let config = control_context.lock().unwrap().setup().clone();
    let (vit_controller, vit_parameters) = VitBackendSettingsBuilder::default()
        .config(&config)
        .session_settings(network_params.session_settings())
        .build()?;

    let hersir_monitor_controller = hersir::controller::MonitorController::new(
        vit_controller.hersir_controller(),
        network_params.session_settings(),
    )?;
    let mut monitor_controller = MonitorController::new(vit_controller, hersir_monitor_controller);

    monitor_controller.monitor_nodes();

    let mut nodes_list = vec![];
    for spawn_param in network_params.nodes_params() {
        nodes_list.push(monitor_controller.spawn_node(spawn_param)?);
    }

    let vit_station = monitor_controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        config.service.version,
    )?;
    let mut wallet_proxy =
        monitor_controller.spawn_wallet_proxy_custom(&mut network_params.proxy_params())?;

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

            monitor_controller.finalize();
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
