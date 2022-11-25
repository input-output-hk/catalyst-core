use crate::snapshot_wormhole::Config;
use color_eyre::eyre;
use color_eyre::eyre::Result;
use job_scheduler_ng::{Job, JobScheduler};
use jormungandr_automation::jormungandr::LogLevel;
use jortestkit::prelude::WaitBuilder;
use snapshot_lib::RawSnapshot;
use snapshot_trigger_service::client::SnapshotResult;
use snapshot_trigger_service::{client::rest::SnapshotRestClient, config::JobParameters};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use thiserror::Error;
use tracing::{debug, error, info, instrument, level_filters::LevelFilter};
use tracing_subscriber::FmtSubscriber;
use vit_servicing_station_lib::v0::endpoints::snapshot::RawSnapshotInput;
use vit_servicing_station_tests::common::{
    clients::RestClient, raw_snapshot::RawSnapshot as RawSnapshotRequest,
};
use voting_tools_rs::Output;

#[derive(StructOpt, Debug)]
pub struct SnapshotWormholeCommand {
    #[structopt(short, long)]
    pub config: PathBuf,

    #[structopt(long = "log-level", default_value = "INFO")]
    pub log_level: LogLevel,

    #[structopt(subcommand)]
    cmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    OneShot,
    Schedule(Schedule),
}
impl SnapshotWormholeCommand {
    pub fn exec(self) -> Result<()> {
        color_eyre::install()?;

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
            SubCommand::OneShot => one_shot(&config),
            SubCommand::Schedule(schedule) => schedule.exec(config),
        }
    }
}

#[instrument(fields(
    source=config.snapshot_service.address.to_string(),
    target=config.servicing_station.address.to_string(),
    tag=config.parameters.tag
   ),
   skip(config)
)]
pub fn one_shot(config: &Config) -> Result<(), eyre::Report> {
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

    let snapshot: Vec<Output> = serde_json::from_str(&snapshot_content)?;
    let result = SnapshotResult::from_outputs(snapshot_jobs_status, snapshot)?;
    debug!("snapshot parsed.");

    let raw_snapshot_input = RawSnapshotRequest {
        tag: config.parameters.tag.clone(),
        content: RawSnapshotInput {
            snapshot: RawSnapshot::from(result.registrations().clone()),
            update_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
            min_stake_threshold: config.parameters.min_stake_threshold,
            voting_power_cap: config.parameters.voting_power_cap,
            direct_voters_group: config.parameters.direct_voters_group.clone(),
            representatives_group: config.parameters.representatives_group.clone(),
        },
    };

    debug!("updating vit servicing station server...");
    let vit_ss_rest_client = RestClient::new(config.servicing_station.address.parse()?);
    vit_ss_rest_client.put_raw_snapshot(&raw_snapshot_input)?;
    info!("transfer done.");
    Ok(())
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Config, eyre::Report> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

#[derive(StructOpt, Debug)]
pub struct Schedule {
    #[structopt(long)]
    pub cron: String,

    #[structopt(short, long)]
    pub eagerly: bool,
}

impl Schedule {
    pub fn exec(self, config: Config) -> Result<(), eyre::Report> {
        if self.eagerly {
            self.run_single(&config);
        }
        let mut sched = JobScheduler::new();

        sched.add(Job::new(self.cron.parse()?, || self.run_single(&config)));

        loop {
            sched.tick();
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    pub fn run_single(&self, config: &Config) {
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
