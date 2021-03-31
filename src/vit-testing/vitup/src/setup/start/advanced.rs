use crate::manager::ControlContext;
use crate::manager::ManagerService;
use crate::scenario::network::single_run;
use crate::scenario::network::{endless_mode, interactive_mode, setup_network};
use crate::setup::generate::read_config;
use crate::setup::start::quick::parse_mode_from_str;
use crate::setup::start::quick::Mode;
use crate::setup::start::QuickVitBackendSettingsBuilder;
use crate::Result;
use jormungandr_scenario_tests::programs::prepare_command;
use jormungandr_scenario_tests::{Context, Seed};
use jortestkit::prelude::{parse_progress_bar_mode_from_str, ProgressBarMode};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use structopt::StructOpt;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct AdvancedStartCommandArgs {
    /// path or name of the jormungandr node to test
    #[structopt(long = "jormungandr", default_value = "jormungandr")]
    pub jormungandr: PathBuf,

    /// path or name of the jcli to test
    #[structopt(long = "jcli", default_value = "jcli")]
    pub jcli: PathBuf,

    /// set a directory in which the tests will be run, allowing every details
    /// to be save persistently. By default it will create temporary directories
    /// and will delete the files and documents
    #[structopt(long = "root-dir", default_value = ".")]
    pub testing_directory: PathBuf,

    /// in some circumstances progress bar can spoil test logs (e.g. on test build job)
    /// if this parametrer value is true, then no progress bar is visible,
    /// but simple log on console enabled
    ///
    /// no progress bar, only simple console output
    #[structopt(
        long = "progress-bar-mode",
        default_value = "Monitor",
        parse(from_str = parse_progress_bar_mode_from_str)
    )]
    pub progress_bar_mode: ProgressBarMode,

    /// to set if to reproduce an existing test
    #[structopt(long = "seed")]
    pub seed: Option<Seed>,

    /// level for all nodes
    #[structopt(long = "log-level", default_value = "info")]
    pub log_level: String,

    /// interactive mode introduce easy way to interact with backend
    /// is capable of quering logs, sending transactions (e.g. tallying), etc.,
    #[structopt(
        long = "mode",
        default_value = "Endless",
        parse(from_str = parse_mode_from_str)
    )]
    pub mode: Mode,

    /// endopint in format: 127.0.0.1:80
    #[structopt(long = "endpoint", default_value = "0.0.0.0:80")]
    pub endpoint: String,

    /// token, only applicable if service mode is used
    #[structopt(long = "token")]
    pub token: Option<String>,

    /// how many qr to generate
    #[structopt(long = "config")]
    pub config: PathBuf,

    /// proposals import json
    #[structopt(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges import json
    #[structopt(
        long = "challenges",
        default_value = "../resources/external/challenges.json"
    )]
    pub challenges: PathBuf,

    /// funds import json
    #[structopt(long = "funds", default_value = "../resources/external/funds.json")]
    pub funds: PathBuf,

    #[structopt(long = "snapshot")]
    pub snapshot: Option<PathBuf>,
}

impl AdvancedStartCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let jormungandr = prepare_command(&self.jormungandr);
        let jcli = prepare_command(&self.jcli);
        let mut progress_bar_mode = self.progress_bar_mode;
        let seed = self
            .seed
            .unwrap_or_else(|| Seed::generate(rand::rngs::OsRng));
        let mut testing_directory = self.testing_directory;
        let generate_documentation = true;
        let log_level = self.log_level;
        let mode = self.mode;
        let endpoint = self.endpoint;
        let token = self.token;

        if mode == Mode::Interactive {
            progress_bar_mode = ProgressBarMode::None;
        }

        let context = Context::new(
            seed,
            jormungandr,
            jcli,
            Some(testing_directory.clone()),
            generate_documentation,
            progress_bar_mode,
            log_level,
        );

        let config = read_config(&self.config)?;

        println!("{:?}", config.params);

        let mut quick_setup = QuickVitBackendSettingsBuilder::new();
        quick_setup.upload_parameters(config.params.clone());
        quick_setup.fees(config.linear_fees);
        quick_setup.set_external_committees(config.committees);

        let mut template_generator =
            ExternalValidVotingTemplateGenerator::new(self.proposals, self.challenges, self.funds)
                .unwrap();

        testing_directory.push(quick_setup.title());
        if testing_directory.exists() {
            std::fs::remove_dir_all(&testing_directory)?;
        }
        match mode {
            Mode::Service => {
                let protocol = quick_setup.protocol().clone();

                let control_context = Arc::new(Mutex::new(ControlContext::new(
                    testing_directory.clone(),
                    quick_setup.parameters().clone(),
                    token,
                )));

                let mut manager = ManagerService::new(control_context.clone());
                manager.spawn();

                loop {
                    if manager.request_to_start() {
                        if testing_directory.exists() {
                            std::fs::remove_dir_all(testing_directory.clone())?;
                        }

                        let mut generator = template_generator.clone();

                        let parameters = manager.setup();
                        quick_setup.upload_parameters(parameters);
                        manager.clear_requests();
                        single_run(
                            control_context.clone(),
                            context.clone(),
                            quick_setup.clone(),
                            endpoint.clone(),
                            &protocol,
                            &mut generator,
                        )?;
                    }

                    std::thread::sleep(std::time::Duration::from_secs(30));
                }
            }
            Mode::Endless => {
                let (mut vit_controller, mut controller, vit_parameters, version) =
                    quick_setup.build(context)?;
                let (_nodes_list, _vit_station, _wallet_proxy) = setup_network(
                    &mut controller,
                    &mut vit_controller,
                    vit_parameters,
                    &mut template_generator,
                    endpoint,
                    quick_setup.protocol(),
                    version,
                )?;
                endless_mode()?;
            }
            Mode::Interactive => {
                let (mut vit_controller, mut controller, vit_parameters, version) =
                    quick_setup.build(context)?;

                let (nodes_list, vit_station, wallet_proxy) = setup_network(
                    &mut controller,
                    &mut vit_controller,
                    vit_parameters,
                    &mut template_generator,
                    endpoint,
                    quick_setup.protocol(),
                    version,
                )?;
                interactive_mode(controller, nodes_list, vit_station, wallet_proxy)?;
            }
        }
        Ok(())
    }
}
