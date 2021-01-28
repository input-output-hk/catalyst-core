use super::QuickVitBackendSettingsBuilder;
use crate::scenario::network::service_mode;
use crate::scenario::network::{endless_mode, interactive_mode, setup_network};
use crate::setup::quick::mode::parse_mode_from_str;
use crate::setup::{initials::Initials, quick::Mode};
use crate::Result;
use jormungandr_scenario_tests::programs::prepare_command;
use jormungandr_scenario_tests::{Context, Seed};
use jortestkit::prelude::read_file;
use jortestkit::prelude::{parse_progress_bar_mode_from_str, ProgressBarMode};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct QuickStartCommandArgs {
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

    /// how many addresses to generate
    #[structopt(long = "initials")]
    pub initials: Option<usize>,

    /// json file which define funds for each account
    /// example:
    /// {
    ///   "8000",
    ///   "10000",
    /// }
    #[structopt(long = "initials-mapping")]
    pub initials_mapping: Option<PathBuf>,

    /// vote start epoch of vote plan
    #[structopt(long = "vote-start-epoch", default_value = "1")]
    pub vote_start_epoch: u32,

    /// vote start epoch of vote plan
    #[structopt(long = "tally-start-epoch", default_value = "2")]
    pub tally_start_epoch: u32,

    /// vote tally end epoch of vote plan
    #[structopt(long = "tally-end-epoch", default_value = "3")]
    pub tally_end_epoch: u32,

    #[structopt(long = "vote-start-timestamp")]
    pub vote_start_timestamp: Option<String>,

    /// vote start epoch of vote plan
    #[structopt(long = "tally-start-timestamp")]
    pub tally_start_timestamp: Option<String>,

    /// vote tally end epoch of vote plan
    #[structopt(long = "tally-end-timestamp")]
    pub tally_end_timestamp: Option<String>,

    /// vote tally end epoch of vote plan
    #[structopt(long = "next-vote-timestamp")]
    pub next_vote_timestamp: Option<String>,

    /// slot duration
    #[structopt(long = "slot-duration", default_value = "20")]
    pub slot_duration: u8,

    /// slots in epoch
    #[structopt(long = "slots-in-epoch", default_value = "60")]
    pub slots_in_epoch: u32,

    /// proposals number
    #[structopt(long = "proposals", default_value = "10")]
    pub proposals: u32,

    /// voting power threshold for participating in voting
    #[structopt(long = "voting-power", default_value = "8000")]
    pub voting_power: u64,

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

    /// switch to private voting type
    #[structopt(long = "private")]
    pub private: bool,
}

impl QuickStartCommandArgs {
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

        let mut quick_setup = QuickVitBackendSettingsBuilder::new();

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping);
            let initials: Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            quick_setup.initials(initials);
        } else if let Some(initials_count) = self.initials {
            quick_setup.initials_count(initials_count, "1234");
        }

        let vote_timestamps = vec![
            self.vote_start_timestamp.clone(),
            self.tally_start_timestamp.clone(),
            self.tally_end_timestamp.clone(),
        ];

        let vote_timestamps_defined = vote_timestamps.iter().filter(|x| x.is_some()).count();
        if vote_timestamps_defined < 3 && vote_timestamps_defined > 0 {
            panic!("either define all voting dates or none");
        }

        quick_setup
            .vote_start_epoch(self.vote_start_epoch)
            .tally_start_epoch(self.tally_start_epoch)
            .tally_end_epoch(self.tally_end_epoch)
            .vote_start_timestamp(self.vote_start_timestamp)
            .tally_start_timestamp(self.tally_start_timestamp)
            .tally_end_timestamp(self.tally_end_timestamp)
            .next_vote_timestamp(self.next_vote_timestamp)
            .slot_duration_in_seconds(self.slot_duration)
            .slots_in_epoch_count(self.slots_in_epoch)
            .proposals_count(self.proposals)
            .voting_power(self.voting_power)
            .private(self.private);

        jormungandr_scenario_tests::introduction::print(&context, "VOTING BACKEND");

        testing_directory.push(quick_setup.title());
        if testing_directory.exists() {
            std::fs::remove_dir_all(&testing_directory)?;
        }
        match mode {
            Mode::Service => service_mode(context, testing_directory, quick_setup, endpoint)?,
            Mode::Endless => {
                let (mut vit_controller, mut controller, vit_parameters) =
                    quick_setup.build(context)?;
                let (_nodes_list, _vit_station, _wallet_proxy) = setup_network(
                    &mut controller,
                    &mut vit_controller,
                    vit_parameters,
                    endpoint,
                )?;
                endless_mode()?;
            }
            Mode::Interactive => {
                let (mut vit_controller, mut controller, vit_parameters) =
                    quick_setup.build(context)?;
                let (nodes_list, vit_station, wallet_proxy) = setup_network(
                    &mut controller,
                    &mut vit_controller,
                    vit_parameters,
                    endpoint,
                )?;
                interactive_mode(controller, nodes_list, vit_station, wallet_proxy)?;
            }
        }
        Ok(())
    }
}
