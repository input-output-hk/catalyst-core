use crate::{
    config::{read_config, Configuration},
    service::ManagerService,
    Context,
};
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
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
    pub fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;

        if self.token.is_some() {
            configuration.token = self.token;
        }

        let control_context = Arc::new(Mutex::new(Context::new(
            configuration.clone(),
            &configuration.result_dir,
        )));

        let mut manager = ManagerService::new(control_context.clone());
        manager.spawn();

        loop {
            if let Some((job_id, params)) = manager.request_to_start() {
                let mut job_result_dir = configuration.result_dir.clone();
                job_result_dir.push(job_id.to_string());
                std::fs::create_dir_all(job_result_dir.clone())?;

                let mut child = configuration.spawn_command(job_id, params).unwrap();

                control_context.lock().unwrap().run_started().unwrap();

                child.wait().unwrap();
                control_context.lock().unwrap().run_finished().unwrap();
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot spawn configuration")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot read configuration")]
    CannotReadConfiguration(#[from] crate::config::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
}
