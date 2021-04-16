use crate::load::IapyxLoad;
use crate::load::IapyxLoadConfig;
use crate::load::IapyxLoadError;
use crate::load::MultiControllerError;
pub use jortestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use jortestkit::load::{Configuration, Monitor};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IapyxLoadCommandError {
    #[error("duration or requests per thread stategy has to be defined")]
    NoStrategyDefined,
    #[error("load runner error")]
    IapyxLoadError(#[from] IapyxLoadError),
    #[error("internal error")]
    MultiControllerError(#[from] MultiControllerError),
}

#[derive(StructOpt, Debug)]
pub struct IapyxLoadCommand {
    /// Prints nodes related data, like stats,fragments etc.
    #[structopt(short = "t", long = "threads", default_value = "3")]
    pub threads: usize,
    /// address in format:
    /// 127.0.0.1:8000
    #[structopt(short = "a", long = "address", default_value = "127.0.0.1:8000")]
    pub address: String,

    /// amount of delay [miliseconds] between requests
    #[structopt(short = "p", long = "pace", default_value = "10")]
    pub pace: u64,

    // duration of scenario
    #[structopt(long = "duration")]
    pub duration: Option<u64>,

    /// how many requests per thread should be sent
    #[structopt(short = "n", long = "requests-per-thread")]
    pub count: Option<u32>,

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
    pub use_https_for_post: bool,

    /// use https for sending fragments
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// use https for sending fragments
    #[structopt(short = "r", long = "reuse_accounts")]
    pub reuse_accounts: bool,

    #[structopt(long = "status-pace", default_value = "1")]
    pub status_pace: u64,

    // measure
    #[structopt(short = "c", long = "criterion")]
    pub criterion: Option<u8>,

    // show progress
    #[structopt(
        long = "progress-bar-mode",
        short = "b",
        default_value = "Monitor",
        parse(from_str = parse_progress_bar_mode_from_str)
    )]
    progress_bar_mode: ProgressBarMode,
}

impl IapyxLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        let config = self.build_config()?;
        let iapyx_load = IapyxLoad::new(config);
        if let Some(stats) = iapyx_load.start()? {
            stats.print()
        }
        Ok(())
    }

    fn build_monitor(&self) -> Monitor {
        match self.progress_bar_mode {
            ProgressBarMode::Monitor => Monitor::Progress(100),
            ProgressBarMode::Standard => Monitor::Standard(100),
            ProgressBarMode::None => Monitor::Disabled(10),
        }
    }

    fn build_config(&self) -> Result<IapyxLoadConfig, IapyxLoadCommandError> {
        let config = if let Some(duration) = self.duration {
            Configuration::duration(
                self.threads,
                std::time::Duration::from_secs(duration),
                self.pace,
                self.build_monitor(),
                0,
                self.status_pace,
            )
        } else if let Some(count) = self.count {
            Configuration::requests_per_thread(
                self.threads,
                count,
                self.pace,
                self.build_monitor(),
                0,
                self.status_pace,
            )
        } else {
            return Err(IapyxLoadCommandError::NoStrategyDefined);
        };

        Ok(IapyxLoadConfig {
            config,
            criterion: self.criterion,
            address: self.address.clone(),
            wallet_mnemonics_file: self.wallet_mnemonics_file.clone(),
            qr_codes_folder: self.qr_codes_folder.clone(),
            reuse_accounts: self.reuse_accounts,
            secrets_folder: self.secrets_folder.clone(),
            global_pin: self.global_pin.clone(),
            read_pin_from_filename: self.read_pin_from_filename,
            use_https_for_post: self.use_https_for_post,
            debug: self.debug,
        })
    }
}
