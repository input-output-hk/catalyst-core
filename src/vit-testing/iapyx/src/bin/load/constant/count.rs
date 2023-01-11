use crate::load::build_monitor;
use crate::load::IapyxLoadCommandError;
use iapyx::NodeLoad;
use iapyx::NodeLoadConfig;
pub use jortestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use jortestkit::load::ConfigurationBuilder;
use std::path::PathBuf;
use std::time::Duration;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct ConstantCountIapyxLoadCommand {
    /// Prints nodes related data, like stats,fragments etc.
    #[clap(short = 't', long = "threads", default_value = "3")]
    pub threads: usize,

    /// Address in format:
    /// 127.0.0.1:8000
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:8000")]
    pub address: String,

    /// Amount of delay (in miliseconds) between requests
    #[clap(short = 'd', long = "delay", default_value = "10000")]
    pub delay: u64,

    /// How many requests per thread should be sent
    #[clap(short = 'n', long = "requests-per-thread")]
    pub count: u32,

    /// Qr codes source folder
    #[clap(short = 'q', long = "qr-codes-folder")]
    pub qr_codes_folder: Option<PathBuf>,

    /// Secrets source folder
    #[clap(short = 's', long = "secrets-folder")]
    pub secrets_folder: Option<PathBuf>,

    /// Global pin for all qr codes
    #[clap(long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    /// Read pin from filename of each qr code
    #[clap(long = "read-from-filename")]
    pub read_pin_from_filename: bool,

    /// Use https for sending fragments
    #[clap(short = 'h', long = "https")]
    pub use_https: bool,

    /// Print additional information
    #[clap(long = "debug")]
    pub debug: bool,

    /// Update all accounts state before sending any vote
    #[clap(long = "reuse-accounts-early")]
    pub reuse_accounts_early: bool,

    /// Update account state just before sending vote
    #[clap(long = "reuse-accounts-lazy")]
    pub reuse_accounts_lazy: bool,

    /// How frequent (in seconds) to print status
    #[clap(long = "status-pace", default_value = "1")]
    pub status_pace: u64,

    /// Pass criteria
    #[clap(short = 'c', long = "criterion")]
    pub criterion: Option<u8>,

    /// Show progress. Available are (Monitor,Standard,None)
    #[clap(
        long = "progress-bar-mode",
        short = 'b',
        default_value = "Monitor",
        value_parser = parse_progress_bar_mode_from_str
    )]
    progress_bar_mode: ProgressBarMode,

    #[clap(default_value = "direct", long)]
    pub voting_group: String,
}

impl ConstantCountIapyxLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        let config = self.build_config();
        let iapyx_load = NodeLoad::new(config);
        if let Some(stats) = iapyx_load.start()? {
            stats.print()
        }
        Ok(())
    }

    fn build_config(&self) -> NodeLoadConfig {
        let config = ConfigurationBuilder::requests_per_thread(self.count)
            .thread_no(self.threads)
            .step_delay(Duration::from_millis(self.delay))
            .fetch_limit(250)
            .monitor(build_monitor(&self.progress_bar_mode))
            .status_pace(Duration::from_secs(self.status_pace))
            .build();

        NodeLoadConfig {
            config,
            use_v1: false,
            batch_size: 1,
            criterion: self.criterion,
            address: self.address.clone(),
            qr_codes_folder: self.qr_codes_folder.clone(),
            reuse_accounts_lazy: self.reuse_accounts_lazy,
            reuse_accounts_early: self.reuse_accounts_early,
            secrets_folder: self.secrets_folder.clone(),
            global_pin: self.global_pin.clone(),
            read_pin_from_filename: self.read_pin_from_filename,
            use_https: self.use_https,
            debug: self.debug,
            voting_group: self.voting_group.clone(),
        }
    }
}
