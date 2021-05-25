use crate::job::VoteRegistrationJobBuilder;
use crate::{
    config::{read_config, Configuration},
    service::ManagerService,
    Context,
};
use futures::future::FutureExt;
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct RegistrationServiceCommand {
    #[structopt(long = "token")]
    pub token: Option<String>,

    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl RegistrationServiceCommand {
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
                if let Some((job_id, request)) = manager.request_to_start() {
                    let mut job_result_dir = configuration.result_dir.clone();
                    job_result_dir.push(job_id.to_string());
                    std::fs::create_dir_all(job_result_dir.clone())?;

                    let job = VoteRegistrationJobBuilder::new()
                        .with_jcli(&configuration.jcli)
                        .with_cardano_cli(&configuration.cardano_cli)
                        .with_voter_registration(&configuration.voter_registration)
                        .with_network(configuration.network)
                        .with_kedqr(&configuration.vit_kedqr)
                        .with_working_dir(&job_result_dir)
                        .build();

                    control_context.lock().unwrap().run_started().unwrap();
                    let output_info = job.start(request).unwrap();
                    control_context
                        .lock()
                        .unwrap()
                        .run_finished(output_info)
                        .unwrap();
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

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot spawn configuration")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot read configuration")]
    CannotReadConfiguration(#[from] crate::config::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
}
