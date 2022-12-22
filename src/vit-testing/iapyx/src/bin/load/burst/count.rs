use crate::load::{build_monitor, IapyxLoadCommandError};
use iapyx::{NodeLoad, NodeLoadConfig};
pub use jortestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use jortestkit::load::ConfigurationBuilder;
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct BurstCountIapyxLoadCommand {
    /// Prints nodes related data, like stats,fragments etc.
    #[structopt(short = "t", long = "threads", default_value = "3")]
    pub threads: usize,
    /// Address in format:
    /// 127.0.0.1:8000
    #[structopt(short = "a", long = "address", default_value = "127.0.0.1:8000")]
    pub address: String,

    /// Amount of delay (in miliseconds) between requests
    #[structopt(short = "d", long = "delay", default_value = "10000")]
    pub delay: u64,

    /// Number of votes sent in single batch
    #[structopt(short = "b", long = "batch-size", default_value = "100")]
    pub batch_size: usize,

    /// How many requests per thread should be sent
    #[structopt(short = "n", long = "bursts-count")]
    pub count: u32,

    /// Qr codes source folder
    #[structopt(short = "q", long = "qr-codes-folder")]
    pub qr_codes_folder: Option<PathBuf>,

    /// Secrets source folder
    #[structopt(short = "s", long = "secrets-folder")]
    pub secrets_folder: Option<PathBuf>,

    /// Global pin for all qr codes
    #[structopt(long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    /// Read pin from filename of each qr code
    #[structopt(long = "read-from-filename")]
    pub read_pin_from_filename: bool,

    /// Use https for sending fragments
    #[structopt(short = "h", long = "https")]
    pub use_https: bool,

    /// Print additional information
    #[structopt(long = "debug")]
    pub debug: bool,

    /// Update all accounts state before sending any vote
    #[structopt(long = "reuse-accounts-early")]
    pub reuse_accounts_early: bool,

    /// Update account state just before sending vote
    #[structopt(long = "reuse-accounts-lazy")]
    pub reuse_accounts_lazy: bool,

    /// How frequent (in seconds) to print status
    #[structopt(long = "status-pace", default_value = "1")]
    pub status_pace: u64,

    /// Pass criteria
    #[structopt(short = "c", long = "criterion")]
    pub criterion: Option<u8>,

    /// Use v1 endpoint for sending votes
    #[structopt(long = "v1")]
    pub use_v1: bool,

    /// Show progress. Available are (Monitor,Standard,None)
    #[structopt(
        long = "progress-bar-mode",
        default_value = "Monitor",
        parse(from_str = parse_progress_bar_mode_from_str)
    )]
    progress_bar_mode: ProgressBarMode,

    #[structopt(default_value = "direct", long)]
    pub voting_group: String,
}

impl BurstCountIapyxLoadCommand {
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
            use_v1: self.use_v1,
            batch_size: self.batch_size,
            criterion: self.criterion,
            address: self.address.clone(),
            qr_codes_folder: self.qr_codes_folder.clone(),
            reuse_accounts_early: self.reuse_accounts_early,
            reuse_accounts_lazy: self.reuse_accounts_lazy,
            secrets_folder: self.secrets_folder.clone(),
            global_pin: self.global_pin.clone(),
            read_pin_from_filename: self.read_pin_from_filename,
            use_https: self.use_https,
            debug: self.debug,
            voting_group: self.voting_group.clone(),
        }
    }
}
