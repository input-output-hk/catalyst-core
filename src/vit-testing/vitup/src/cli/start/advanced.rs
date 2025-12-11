use crate::builders::utils::logger;
use crate::config::mode::{parse_mode_from_str, Mode};
use crate::config::read_config;
use crate::config::read_voter_hirs;
use crate::mode::spawn::{spawn_network, NetworkSpawnParams};
use crate::{error::Error, Result};
use chain_addr::Discrimination;
use clap::Parser;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::LogLevel;
use std::path::PathBuf;
use std::str::FromStr;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;

#[derive(Parser, Debug)]
pub struct AdvancedStartCommandArgs {
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

    /// interactive mode introduce easy way to interact with backend
    /// is capable of quering logs, sending transactions (e.g. tallying), etc.,
    #[clap(
        long = "mode",
        default_value = "Endless",
        value_parser = parse_mode_from_str
    )]
    pub mode: Mode,

    /// endopint in format: 127.0.0.1:80
    #[clap(long = "endpoint", default_value = "0.0.0.0:80")]
    pub endpoint: String,

    /// token, only applicable if service mode is used
    #[clap(long = "token")]
    pub token: Option<String>,

    /// how many qr to generate
    #[clap(long = "config")]
    pub config: PathBuf,

    /// proposals import json
    #[clap(
        long = "proposals",
        default_value = "../../catalyst-resources/ideascale/fund6/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges import json
    #[clap(
        long = "challenges",
        default_value = "../../catalyst-resources/ideascale/fund6/challenges.json"
    )]
    pub challenges: PathBuf,

    /// challenges import json
    #[clap(
        long = "reviews",
        default_value = "../../catalyst-resources/ideascale/fund6/reviews.json"
    )]
    pub reviews: PathBuf,

    /// funds import json
    #[clap(
        long = "funds",
        default_value = "../../catalyst-resources/ideascale/fund6/funds.json"
    )]
    pub funds: PathBuf,

    #[clap(long = "snapshot")]
    pub snapshot: Option<PathBuf>,

    #[clap(long = "vitup-log-level", default_value = "info")]
    pub vitup_log_level: LogLevel,
}

impl AdvancedStartCommandArgs {
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
        let title = "advanced";

        let session_settings = SessionSettings {
            jormungandr: jormungandr.to_path_buf(),
            root: testing_directory.join(title).into(),
            generate_documentation,
            mode: mode.into(),
            log: LogLevel::from_str(&log_level)
                .map_err(|_| Error::UnknownLogLevel(log_level.clone()))?,
            title: title.to_owned(),
        };

        let mut config = read_config(&self.config)?;

        if let Some(snapshot) = self.snapshot {
            config
                .initials
                .block0
                .extend_from_external(read_voter_hirs(snapshot)?, Discrimination::Production);
        }

        let mut template_generator = ExternalValidVotingTemplateGenerator::new(
            self.proposals,
            self.challenges,
            self.funds,
            self.reviews,
        )?;

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
        spawn_network(mode, network_spawn_params, &mut template_generator, config)
    }
}
