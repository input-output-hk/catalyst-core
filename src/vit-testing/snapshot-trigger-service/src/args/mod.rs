use crate::{
    config::{read_config, Configuration},
    context::State,
    service::ManagerService,
    Context,
};
use futures::future::FutureExt;
use std::sync::Mutex;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct TriggerServiceCommand {
    #[structopt(long = "token")]
    pub token: Option<String>,

    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl TriggerServiceCommand {
    pub async fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;

        if self.token.is_some() {
            configuration.token = self.token;
        }

        let control_context = Arc::new(Mutex::new(Context::new(
            configuration.clone(),
            &configuration.result_dir,
        )));

        let mut manager = ManagerService::new(control_context.clone());
        let handle = manager.spawn();

        let request_to_start_task = async {
            loop {
                if let Some((job_id, params)) = manager.request_to_start() {
                    let mut job_result_dir = configuration.result_dir.clone();
                    job_result_dir.push(job_id.to_string());
                    std::fs::create_dir_all(job_result_dir.clone())?;

                    let mut child = configuration.spawn_command(job_id, params).unwrap();

                    control_context.lock().unwrap().run_started().unwrap();

                    child.wait().unwrap();
                    control_context.lock().unwrap().run_finished().unwrap();

                    let status = control_context.lock().unwrap().status_by_id(job_id)?;
                    job_result_dir.push("status.yaml");
                    persist_status(&job_result_dir, status)
                        .map_err(|_| Error::CannotPersistJobState)?;
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
        .fuse();

        tokio::pin!(request_to_start_task);

        futures::select! {
            result = request_to_start_task => result,
            _ = handle.fuse() => Ok(()),
        }
    }
}

pub fn persist_status<P: AsRef<Path>>(path: P, state: State) -> Result<(), Error> {
    use std::io::Write;
    let content = serde_yaml::to_string(&state)?;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot spawn configuration")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot read configuration")]
    CannotReadConfiguration(#[from] crate::config::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
    #[error("cannot persist job state")]
    CannotPersistJobState,
    #[error("cannot serialize job state")]
    CannotSerializeJobState(#[from] serde_yaml::Error),
}
