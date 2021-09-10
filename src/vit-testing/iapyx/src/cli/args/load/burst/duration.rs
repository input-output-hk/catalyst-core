use crate::cli::args::load::build_monitor;
use crate::cli::args::load::IapyxLoadCommandError;
use crate::load::NodeLoad;
use crate::load::NodeLoadConfig;
pub use jortestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use jortestkit::load::Configuration;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct BurstDurationIapyxLoadCommand {
    /// Prints nodes related data, like stats,fragments etc.
    #[structopt(short = "t", long = "threads", default_value = "3")]
    pub threads: usize,
    /// address in format:
    /// 127.0.0.1:8000
    #[structopt(short = "a", long = "address", default_value = "127.0.0.1:8000")]
    pub address: String,

    /// amount of delay [miliseconds] between requests
    #[structopt(short = "p", long = "pace", default_value = "10000")]
    pub pace: u64,

    /// amount of delay [miliseconds] between requests
    #[structopt(short = "b", long = "batch-size", default_value = "100")]
    pub batch_size: usize,

    // use v1 endpoint for bursts
    #[structopt(long = "v1")]
    pub use_v1: bool,

    // duration of scenario
    #[structopt(long = "duration")]
    pub duration: u64,

    /// wallet mnemonics file
    #[structopt(long = "mnemonics")]
    pub wallet_mnemonics_file: Option<PathBuf>,

    #[structopt(short = "q", long = "qr-codes-folder")]
    pub qr_codes_folder: Option<PathBuf>,

    #[structopt(short = "s", long = "secrets-folder")]
    pub secrets_folder: Option<PathBuf>,

    #[structopt(long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    #[structopt(long = "read-from-filename")]
    pub read_pin_from_filename: bool,

    /// use https for sending fragments
    #[structopt(short = "h", long = "https")]
    pub use_https: bool,

    /// use https for sending fragments
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// update all accounts state before sending any vote
    #[structopt(long = "reuse-accounts-early")]
    pub reuse_accounts_early: bool,

    /// update account state just before sending vote
    #[structopt(long = "reuse-accounts-lazy")]
    pub reuse_accounts_lazy: bool,

    #[structopt(long = "status-pace", default_value = "1")]
    pub status_pace: u64,

    // measure
    #[structopt(short = "c", long = "criterion")]
    pub criterion: Option<u8>,

    // show progress
    #[structopt(
        long = "progress-bar-mode",
        default_value = "Monitor",
        parse(from_str = parse_progress_bar_mode_from_str)
    )]
    progress_bar_mode: ProgressBarMode,
}

impl BurstDurationIapyxLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        let config = self.build_config();
        let iapyx_load = NodeLoad::new(config);
        if let Some(stats) = iapyx_load.start()? {
            stats.print()
        }
        Ok(())
    }

    fn build_config(&self) -> NodeLoadConfig {
        let config = Configuration::duration(
            self.threads,
            std::time::Duration::from_secs(self.duration),
            self.pace,
            Some(250),
            build_monitor(&self.progress_bar_mode),
            0,
            self.status_pace,
        );

        NodeLoadConfig {
            config,
            use_v1: self.use_v1,
            batch_size: self.batch_size,
            criterion: self.criterion,
            address: self.address.clone(),
            wallet_mnemonics_file: self.wallet_mnemonics_file.clone(),
            qr_codes_folder: self.qr_codes_folder.clone(),
            reuse_accounts_early: self.reuse_accounts_early,
            reuse_accounts_lazy: self.reuse_accounts_lazy,
            secrets_folder: self.secrets_folder.clone(),
            global_pin: self.global_pin.clone(),
            read_pin_from_filename: self.read_pin_from_filename,
            use_https: self.use_https,
            debug: self.debug,
        }
    }
}
