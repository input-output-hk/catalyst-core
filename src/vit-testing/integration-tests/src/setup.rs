use jormungandr_scenario_tests::prepare_command;
use jormungandr_scenario_tests::scenario::Controller;
use jormungandr_scenario_tests::Context;
use jormungandr_testing_utils::testing::network_builder::Seed;
use jortestkit::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vitup::scenario::controller::VitController;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;

pub fn wait_until_folder_contains_all_qrs<P: AsRef<Path>>(qrs_count: usize, folder: P) {
    println!("waiting for qr code in: {:?}", folder.as_ref());

    loop {
        let qrs = std::fs::read_dir(folder.as_ref()).unwrap();
        let actual = qrs.into_iter().count();
        println!("waiting for qr code in: {}/{}", actual, qrs_count);
        if actual >= qrs_count {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

pub fn context(testing_directory: &PathBuf) -> Context {
    let jormungandr = prepare_command(PathBuf::from_str("jormungandr").unwrap());
    let jcli = prepare_command(PathBuf::from_str("jcli").unwrap());
    let seed = Seed::generate(rand::rngs::OsRng);
    let generate_documentation = true;
    let log_level = "info".to_string();

    Context::new(
        seed,
        jormungandr,
        jcli,
        Some(testing_directory.clone()),
        generate_documentation,
        ProgressBarMode::None,
        log_level,
    )
}

pub fn vitup_setup_default(
    private: bool,
    no_of_wallets: usize,
    testing_directory: PathBuf,
) -> (VitController, Controller, ValidVotePlanParameters, String) {
    let mut quick_setup = QuickVitBackendSettingsBuilder::new();

    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_start_epoch(0)
        .tally_start_epoch(20)
        .tally_end_epoch(21)
        .slot_duration_in_seconds(5)
        .slots_in_epoch_count(10)
        .proposals_count(100)
        .voting_power(8_000)
        .private(private);

    vitup_setup(quick_setup, testing_directory)
}

pub fn vitup_setup(
    mut quick_setup: QuickVitBackendSettingsBuilder,
    mut testing_directory: PathBuf,
) -> (VitController, Controller, ValidVotePlanParameters, String) {
    let context = context(&testing_directory);

    testing_directory.push(quick_setup.title());
    if testing_directory.exists() {
        std::fs::remove_dir_all(&testing_directory).unwrap();
    }

    let fund_name = quick_setup.fund_name();
    let (vit_controller, controller, vit_parameters) = quick_setup.build(context).unwrap();
    (vit_controller, controller, vit_parameters, fund_name)
}
