mod config;

use crate::load::MultiControllerError;
use crate::qr::PinReadMode;
use crate::VoteStatusProvider;
use crate::{MultiController, WalletRequestGen};
pub use config::IapyxLoadConfig;
use jormungandr_testing_utils::testing::node::RestSettings;
pub use jortestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use jortestkit::load;
use jortestkit::load::{Configuration, Monitor};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IapyxLoadCommandError {
    #[error("duration or requests per thread stategy has to be defined")]
    NoStrategyDefined,
    #[error("cannot read mnemonics file")]
    CannotReadMnemonicsFile,
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
    #[structopt(short = "r", long = "duration")]
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

    // measure
    #[structopt(short = "m", long = "measure")]
    pub measure: bool,

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

        let backend = config.address;

        let settings = RestSettings {
            enable_debug: self.debug,
            use_https_for_post: self.use_https_for_post,
            ..Default::default()
        };

        let pin_read_mode = PinReadMode::new(self.read_pin_from_filename, &self.global_pin);

        println!("{:?}", settings);

        let multicontroller = {
            if let Some(mnemonics_file) = &self.wallet_mnemonics_file {
                let mnemonics = jortestkit::file::read_file_as_vector(&mnemonics_file)
                    .map_err(|_e| IapyxLoadCommandError::CannotReadMnemonicsFile)?;

                MultiController::recover(&backend, mnemonics, &[], settings)
            } else if let Some(qr_codes) = &self.qr_codes_folder {
                let qr_codes: Vec<PathBuf> = std::fs::read_dir(qr_codes)
                    .unwrap()
                    .into_iter()
                    .map(|x| x.unwrap().path())
                    .collect();

                MultiController::recover_from_qrs(&backend, &qr_codes, pin_read_mode, settings)
            } else if let Some(secrets_folder) = &self.secrets_folder {
                let secrets: Vec<PathBuf> = std::fs::read_dir(secrets_folder)
                    .unwrap()
                    .into_iter()
                    .map(|x| x.unwrap().path())
                    .collect();
                MultiController::recover_from_sks(&backend, &secrets, settings)
            } else {
                panic!("source of private keys not selected");
            }
        };

        let mut request_generator = WalletRequestGen::new(multicontroller?);
        request_generator.fill_generator().unwrap();

        load::start_async(
            request_generator,
            VoteStatusProvider::new(backend),
            config.config,
            "Wallet backend load test",
        );
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
            )
        } else if let Some(count) = self.count {
            Configuration::requests_per_thread(
                self.threads,
                count,
                self.pace,
                self.build_monitor(),
                0,
            )
        } else {
            return Err(IapyxLoadCommandError::NoStrategyDefined);
        };

        Ok(IapyxLoadConfig::new(
            config,
            self.measure,
            self.address.clone(),
        ))
    }
}
