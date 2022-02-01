use crate::builders::utils::SessionSettingsExtension;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::VoteBlockchainTime;
use crate::scenario::controller::VitController;
use crate::scenario::spawn::NetworkSpawnParams;
use crate::vit_station::ValidVotePlanParameters;
use crate::vit_station::ValidVotingTemplateGenerator;
use crate::vit_station::VitStationController;
use crate::wallet::WalletProxyController;
use crate::Result;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::JormungandrProcess;
use std::path::PathBuf;

pub fn vitup_setup_default(
    private: bool,
    no_of_wallets: usize,
    testing_directory: PathBuf,
) -> (
    VitController,
    ValidVotePlanParameters,
    NetworkSpawnParams,
    String,
) {
    let mut quick_setup = VitBackendSettingsBuilder::new();

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 20,
        tally_end: 21,
        slots_per_epoch: 10,
    };

    quick_setup
        .initials_count(no_of_wallets, "1234")
        .slot_duration_in_seconds(5)
        .vote_timing(vote_timing.into())
        .proposals_count(100)
        .voting_power(8_000)
        .private(private);

    vitup_setup(quick_setup, testing_directory)
}

pub fn vitup_setup(
    mut quick_setup: VitBackendSettingsBuilder,
    mut testing_directory: PathBuf,
) -> (
    VitController,
    ValidVotePlanParameters,
    NetworkSpawnParams,
    String,
) {
    let endpoint = "127.0.0.1:8080";

    let session_settings = SessionSettings::empty_from_dir(&testing_directory);

    testing_directory.push(quick_setup.title());
    if testing_directory.exists() {
        std::fs::remove_dir_all(&testing_directory).unwrap();
    }

    let fund_name = quick_setup.fund_name();
    let (controller, vit_parameters, _) = quick_setup.build(session_settings.clone()).unwrap();

    let network_spawn_params = NetworkSpawnParams::new(
        endpoint.to_string(),
        quick_setup.parameters(),
        session_settings,
        None,
        testing_directory,
    );

    (controller, vit_parameters, network_spawn_params, fund_name)
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
