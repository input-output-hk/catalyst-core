use crate::builders::VitBackendSettingsBuilder;
use crate::manager::State;
use crate::manager::{ControlContext, ControlContextLock, ManagerService};
use crate::scenario::monitor::MonitorController;
use crate::scenario::spawn::NetworkSpawnParams;
use crate::Result;
use hersir::config::SessionSettings;
use std::sync::Arc;
use std::sync::Mutex;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub fn spawn_network(
    session_settings: SessionSettings,
    network_params: NetworkSpawnParams,
    mut quick_setup: VitBackendSettingsBuilder,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    let protocol = quick_setup.protocol().clone();

    // TODO add failover
    let working_dir = session_settings.root.as_ref().unwrap().to_path_buf();
    working_dir.push(quick_setup.title());

    let control_context = Arc::new(Mutex::new(ControlContext::new(
        working_dir.clone(),
        quick_setup.parameters().clone(),
        network_params.token(),
    )));

    let mut manager = ManagerService::new(control_context.clone());
    manager.spawn();

    loop {
        if manager.request_to_start() {
            if working_dir.exists() {
                std::fs::remove_dir_all(working_dir)?;
            }

            let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

            let parameters = manager.setup();
            quick_setup.upload_parameters(parameters);
            manager.clear_requests();
            single_run(
                control_context.clone(),
                session_settings.clone(),
                quick_setup.clone(),
                network_params,
                &mut template_generator,
            )?;
        }

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

pub fn single_run(
    control_context: ControlContextLock,
    session_settings: SessionSettings,
    mut quick_setup: VitBackendSettingsBuilder,
    network_params: NetworkSpawnParams,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<()> {
    {
        let mut control_context = control_context.lock().unwrap();
        let state = control_context.state_mut();
        *state = State::Starting;
    }

    let (mut vit_controller, vit_parameters, version) =
        quick_setup.build(session_settings.into())?;

    let hersir_monitor_controller = hersir::controller::MonitorController::new(
        vit_controller.hersir_controller(),
        session_settings.clone().into(),
    )?;
    let monitor_controller = MonitorController::new(vit_controller, hersir_monitor_controller);

    monitor_controller.monitor_nodes();

    let nodes_list = vec![];
    for spawn_param in network_params.nodes_params() {
        monitor_controller.spawn_node(spawn_param)?;
    }

    let vit_station = monitor_controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        network_params.version(),
    )?;
    let wallet_proxy =
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
