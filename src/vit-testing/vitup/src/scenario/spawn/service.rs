use crate::builders::VitBackendSettingsBuilder;
use crate::manager::State;
use crate::manager::{ControlContext, ControlContextLock, ManagerService};
use crate::scenario::monitor::MonitorController;
use crate::scenario::spawn::NetworkSpawnParams;
use crate::Result;
use std::sync::Arc;
use std::sync::Mutex;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    network_params: NetworkSpawnParams,
    mut quick_setup: VitBackendSettingsBuilder,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let working_dir = network_params.session_settings().root;
    let control_context = Arc::new(Mutex::new(ControlContext::new(
        working_dir.path(),
        quick_setup.parameters().clone(),
        network_params.token(),
    )));

    let mut manager = ManagerService::new(control_context.clone());
    manager.spawn();

    loop {
        if manager.request_to_start() {
            if working_dir.path().exists() {
                std::fs::remove_dir_all(working_dir.path())?;
            }

            let parameters = manager.setup();
            quick_setup.upload_parameters(parameters);
            manager.clear_requests();
            single_run(
                control_context.clone(),
                quick_setup.clone(),
                network_params.clone(),
                template_generator,
            )?;
        }

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

pub fn single_run(
    control_context: ControlContextLock,
    mut quick_setup: VitBackendSettingsBuilder,
    network_params: NetworkSpawnParams,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    {
        let mut control_context = control_context.lock().unwrap();
        let state = control_context.state_mut();
        *state = State::Starting;
    }

    let (vit_controller, vit_parameters, version) =
        quick_setup.build(network_params.session_settings())?;

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

    let vit_station =
        monitor_controller.spawn_vit_station(vit_parameters, template_generator, version)?;
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
