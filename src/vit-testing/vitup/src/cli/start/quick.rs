use crate::builders::utils::logger;
use crate::config::read_voter_hirs;
use crate::config::ConfigBuilder;
use crate::config::{
    mode::{parse_mode_from_str, Mode},
    Block0Initials as Initials, VoteTime,
};
use crate::mode::spawn::{spawn_network, NetworkSpawnParams};
use crate::{error::Error, Result};
use chain_addr::Discrimination;
use clap::Parser;
use hersir::config::SessionSettings;
use hersir::utils::print_intro;
use jormungandr_automation::jormungandr::LogLevel;
use jortestkit::prelude::read_file;
use std::path::PathBuf;
use std::str::FromStr;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;

#[derive(Parser, Debug)]
pub struct QuickStartCommandArgs {
    /// path or name of the jormungandr node to test
    #[clap(long = "jormungandr", default_value = "jormungandr")]
    pub jormungandr: PathBuf,

    /// path or name of the jcli to test
    #[clap(long = "jcli", default_value = "jcli")]
    pub jcli: PathBuf,

    /// set a directory in which the tests will be run, allowing every details
    /// to be save persistently. By default it will create temporary directories
    /// and will delete the files and documents
    #[clap(long = "root-dir", default_value = "./catalyst")]
    pub testing_directory: PathBuf,

    /// level for all nodes
    #[clap(long = "log-level", default_value = "info")]
    pub log_level: String,

    /// how many addresses to generate
    #[clap(long = "initials")]
    pub initials: Option<usize>,

    /// json file which define funds for each account
    /// example:
    /// {
    ///   "8000",
    ///   "10000",
    /// }
    #[clap(long = "block-initials-mapping", conflicts_with = "initials")]
    pub initials_mapping: Option<PathBuf>,

    /// vote start epoch of vote plan
    #[clap(long = "vote-start-epoch", default_value = "1")]
    pub vote_start_epoch: u32,

    /// vote start epoch of vote plan
    #[clap(long = "tally-start-epoch", default_value = "2")]
    pub tally_start_epoch: u32,

    /// vote tally end epoch of vote plan
    #[clap(long = "tally-end-epoch", default_value = "3")]
    pub tally_end_epoch: u32,

    #[clap(long = "vote-start-timestamp", conflicts_with = "vote_start_epoch")]
    pub vote_start_timestamp: Option<String>,

    /// vote start epoch of vote plan
    #[clap(long = "tally-start-timestamp", conflicts_with = "vote_start_epoch")]
    pub tally_start_timestamp: Option<String>,

    /// vote tally end epoch of vote plan
    #[clap(long = "tally-end-timestamp", conflicts_with = "vote_start_epoch")]
    pub tally_end_timestamp: Option<String>,

    /// vote tally end epoch of vote plan
    #[clap(long = "next-vote-timestamp")]
    pub next_vote_timestamp: Option<String>,

    /// snapshot timestamp
    #[clap(long = "snapshot-timestamp")]
    pub snapshot_timestamp: Option<String>,

    /// slot duration
    #[clap(long = "slot-duration", default_value = "20")]
    pub slot_duration: u8,

    /// slots in epoch
    #[clap(long = "slots-in-epoch", default_value = "60")]
    pub slots_in_epoch: u32,

    /// proposals number
    #[clap(long = "proposals", default_value = "10")]
    pub proposals: u32,

    /// voting power threshold for participating in voting
    #[clap(long = "voting-power", default_value = "8000")]
    pub voting_power: u64,

    /// interactive mode introduce easy way to interact with backend
    /// is capable of quering logs, sending transactions (e.g. tallying), etc.,
    #[clap(
        long = "mode",
        default_value = "Standard",
        value_parser = parse_mode_from_str
    )]
    pub mode: Mode,

    /// endopint in format: 127.0.0.1:80
    #[clap(long = "endpoint", default_value = "0.0.0.0:8080")]
    pub endpoint: String,

    /// switch to private voting type
    #[clap(long = "private")]
    pub private: bool,

    /// use https mode for backend
    #[clap(long = "https")]
    pub https: bool,

    /// switch to private voting type
    #[clap(long = "version", default_value = "2.0")]
    pub version: String,

    /// token, only applicable if service mode is used
    #[clap(long = "token")]
    pub token: Option<String>,

    #[clap(long = "snapshot")]
    pub snapshot: Option<PathBuf>,

    #[clap(long = "vitup-log-level", default_value = "info")]
    pub vitup_log_level: LogLevel,
}

impl QuickStartCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        logger::init(self.vitup_log_level)?;

        let jormungandr = &self.jormungandr;
        let testing_directory = self.testing_directory;
        let generate_documentation = true;
        let log_level = self.log_level;
        let mode = self.mode;
        let endpoint = self.endpoint;
        let token = self.token;
        let title = "quick";

        let session_settings = SessionSettings {
            jormungandr: jormungandr.to_path_buf(),
            root: testing_directory.join(title).into(),
            generate_documentation,
            mode: mode.into(),
            log: LogLevel::from_str(&log_level)
                .map_err(|_| Error::UnknownLogLevel(log_level.clone()))?,
            title: title.to_owned(),
        };

        let mut config_builder = ConfigBuilder::default();

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping)?;
            let initials: Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            config_builder = config_builder.block0_initials(initials);
        } else {
            config_builder =
                config_builder.block0_initials_count(self.initials.unwrap_or(10), "1234");
        }

        if let Some(snapshot) = self.snapshot {
            config_builder = config_builder
                .extend_block0_initials(read_voter_hirs(snapshot)?, Discrimination::Production);
        }

        if self.https {
            config_builder = config_builder.use_https();
        }

        let vote_timestamps = [self.vote_start_timestamp.clone(),
            self.tally_start_timestamp.clone(),
            self.tally_end_timestamp.clone()];

        let vote_timestamps_defined = vote_timestamps.iter().filter(|x| x.is_some()).count();
        if vote_timestamps_defined < 3 && vote_timestamps_defined > 0 {
            panic!("either define all voting dates or none");
        }

        let vote_timing = {
            if self.vote_start_timestamp.is_none() {
                VoteTime::blockchain(
                    self.vote_start_epoch,
                    self.tally_start_epoch,
                    self.tally_end_epoch,
                    self.slots_in_epoch,
                )
            } else {
                VoteTime::real_from_str(
                    self.vote_start_timestamp.unwrap(),
                    self.tally_start_timestamp.unwrap(),
                    self.tally_end_timestamp.unwrap(),
                )?
            }
        };

        let config = config_builder
            .vote_timing(vote_timing)
            .next_vote_timestamp_from_string_if_some(self.next_vote_timestamp)
            .snapshot_timestamp_from_string_if_some(self.snapshot_timestamp)
            .slot_duration_in_seconds(self.slot_duration)
            .proposals_count(self.proposals)
            .voting_power(self.voting_power)
            .private(self.private)
            .version(self.version)
            .build();

        print_intro(&session_settings, "CATALYST BACKEND");

        let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

        if testing_directory.exists() {
            std::fs::remove_dir_all(&testing_directory)?;
        }

        let network_spawn_params = NetworkSpawnParams::new(
            endpoint,
            config.protocol(&testing_directory)?,
            session_settings,
            token,
            config.service.version.clone(),
            testing_directory,
        );
        spawn_network(mode, network_spawn_params, &mut template_generator, config)}
}
