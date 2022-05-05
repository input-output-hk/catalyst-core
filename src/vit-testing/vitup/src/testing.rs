use crate::builders::utils::SessionSettingsExtension;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Config;
use crate::config::ConfigBuilder;
use crate::config::VoteBlockchainTime;
use crate::mode::spawn::NetworkSpawnParams;
use crate::mode::standard::{
    ValidVotePlanParameters, ValidVotingTemplateGenerator, VitController, VitStationController,
    WalletProxyController,
};
use crate::Result;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::JormungandrProcess;
use std::path::PathBuf;

pub fn vitup_setup_default(
    private: bool,
    no_of_wallets: usize,
    testing_directory: PathBuf,
) -> Result<(VitController, ValidVotePlanParameters, NetworkSpawnParams)> {
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 20,
        tally_end: 21,
        slots_per_epoch: 10,
    };

    let config = ConfigBuilder::default()
        .block0_initials_count(no_of_wallets, "1234")
        .slot_duration_in_seconds(5)
        .vote_timing(vote_timing.into())
        .proposals_count(100)
        .voting_power(8_000)
        .private(private)
        .build();

    vitup_setup(&config, testing_directory)
}

pub fn vitup_setup(
    config: &Config,
    testing_directory: PathBuf,
) -> Result<(VitController, ValidVotePlanParameters, NetworkSpawnParams)> {
    let endpoint = "127.0.0.1:8080";

    let session_settings = SessionSettings::from_dir(&testing_directory);

    if testing_directory.exists() {
        std::fs::remove_dir_all(&testing_directory).unwrap();
    }

    let (controller, vit_parameters) = VitBackendSettingsBuilder::default()
        .config(config)
        .session_settings(session_settings.clone())
        .build()?;

    let network_spawn_params = NetworkSpawnParams::new(
        endpoint.to_string(),
        config.protocol(&testing_directory)?,
        session_settings,
        None,
        config.service.version.clone(),
        testing_directory,
    );

    Ok((controller, vit_parameters, network_spawn_params))
}

pub fn spawn_network(
    controller: &mut VitController,
    vit_parameters: ValidVotePlanParameters,
    network_spawn_params: NetworkSpawnParams,
    template_generator: &mut dyn ValidVotingTemplateGenerator,
) -> Result<(
    Vec<JormungandrProcess>,
    VitStationController,
    WalletProxyController,
)> {
    let mut nodes_list = vec![];
    for spawn_param in network_spawn_params.nodes_params() {
        nodes_list.push(controller.spawn_node(spawn_param)?);
    }
    let wallet_proxy =
        controller.spawn_wallet_proxy_custom(&mut network_spawn_params.proxy_params())?;
    let vit_station = controller.spawn_vit_station(
        vit_parameters,
        template_generator,
        network_spawn_params.version(),
    )?;

    Ok((nodes_list, vit_station, wallet_proxy))
}
