pub use crate::snapshot::wormhole::Config;
use crate::snapshot::OutputsExtension;
use color_eyre::eyre::Result;
use job_scheduler_ng::{Job, JobScheduler};
use jormungandr_automation::jormungandr::LogLevel;
use jortestkit::prelude::WaitBuilder;
use snapshot_trigger_service::{client::rest::SnapshotRestClient, config::JobParameters};
use voting_tools_rs::SnapshotEntry;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;
use thiserror::Error;
use tracing::{debug, error, info, instrument, level_filters::LevelFilter};
use tracing_subscriber::FmtSubscriber;
use vit_servicing_station_tests::common::clients::RestClient;

/// Main command
#[derive(StructOpt, Debug)]
pub struct Command {
    /// Path to configuration file
    #[structopt(short, long)]
    pub config: PathBuf,

    /// Log level
    #[structopt(long = "log-level", default_value = "INFO")]
    pub log_level: LogLevel,

    #[structopt(subcommand)]
    cmd: Operation,
}

/// Snapshot wormhole
///
/// Schedules snapshot file transport from snapshot trigger service to
/// servicing station. Two modes are available:
///
/// - one-shot - Ends program after single job is done,
///
/// - schedule - Run job continuously based on cron string.
///              WARNING: there is custom cron string used which allows to program scheduler based on seconds.
///              The scheduling format is as follows:
///
///              sec   min   hour   day of month   month   day of week   year
///
///              *     *     *      *              *       *             *
#[derive(StructOpt, Debug)]
pub enum Operation {
    /// run single job
    OneShot,
    Schedule(Schedule),
}
impl Command {
    /// Executes command
    ///
    /// # Errors
    ///
    /// On IO related errors
    pub fn exec(self) -> Result<()> {
        let subscriber = FmtSubscriber::builder()
            .with_file(false)
            .with_target(true)
            .with_max_level(
                LevelFilter::from_str(self.log_level.as_ref()).expect("invalid log level"),
            )
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;

        let config = read_config(self.config)?;

        match self.cmd {
            Operation::OneShot => one_shot(&config),
            Operation::Schedule(schedule) => schedule.exec(&config),
        }
    }
}

/// Performs one-time snapshot transport operation.
///
/// # Errors
///
/// On any errors from services
#[instrument(fields(
    source=config.snapshot_service.address.to_string(),
    target=config.servicing_station.address.to_string(),
    tag=config.parameters.tag
   ),
   skip(config)
)]
pub fn one_shot(config: &Config) -> Result<()> {
    info!("Job started");

    let snapshot_params = JobParameters {
        slot_no: None,
        tag: Some(config.parameters.tag.clone()),
    };

    let snapshot_client = if let Some(token) = &config.snapshot_service.token {
        debug!("Using token for snapshot retrieval");
        SnapshotRestClient::new_with_token(
            token.to_string(),
            config.snapshot_service.address.to_string(),
        )
    } else {
        debug!("no token for snapshot retrieval");
        SnapshotRestClient::new(config.snapshot_service.address.to_string())
    };

    let snapshot_job_id = snapshot_client.job_new(snapshot_params.clone())?;

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();

    debug!("Awaiting snapshot with strategy: {wait}");
    let snapshot_jobs_status =
        snapshot_client.wait_for_job_finish(snapshot_job_id.clone(), wait)?;

    debug!("Snapshot done with status: {snapshot_jobs_status:?}");
    let snapshot_content =
        snapshot_client.get_snapshot(snapshot_job_id, snapshot_params.tag.unwrap_or_default())?;

    let snapshot: Vec<SnapshotEntry> = serde_json::from_str(&snapshot_content)?;
    let raw_snapshot = snapshot.try_into_raw_snapshot_request(config.parameters.clone())?;
    debug!("snapshot parsed.");

    debug!("updating vit servicing station server...");
    let vit_ss_rest_client = RestClient::new(config.servicing_station.address.parse()?);
    vit_ss_rest_client.put_raw_snapshot(&raw_snapshot)?;
    info!("transfer done.");
    Ok(())
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Config> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

/// Run job in a schedule mode.
#[derive(StructOpt, Debug)]
pub struct Schedule {
    /// Cron string defining frequency of jobs
    #[structopt(long)]
    pub cron: String,

    /// If set to true, job will be run immediately.
    #[structopt(short, long)]
    pub eagerly: bool,
}

impl Schedule {
    /// Executes command
    ///
    /// # Errors
    ///
    /// On Parsing cron job or any services unavailability
    pub fn exec(self, config: &Config) -> Result<()> {
        if self.eagerly {
            Self::run_single(config);
        }
        let mut sched = JobScheduler::new();

        sched.add(Job::new(self.cron.parse()?, || Self::run_single(config)));

        loop {
            sched.tick();
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    fn run_single(config: &Config) {
        if let Err(err) = one_shot(config) {
            error!("scheduled transfer failed due to: {err}");
        }
        info!("waiting for next scheduled run..");
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot create output file")]
    IoError(#[from] std::io::Error),
}
