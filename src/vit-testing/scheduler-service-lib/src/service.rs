use futures::FutureExt;
use std::future::Future;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};
use tokio::runtime::{Handle, Runtime};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct ManagerService {
    runtime: Option<Runtime>,
}

impl Default for ManagerService {
    fn default() -> Self {
        // Do not create a new runtime when already running within a tokio runtime. This is
        // pointless and will result into panic when dropping this structure.
        let runtime = match Handle::try_current() {
            Ok(_) => None,
            Err(_) => Some(Runtime::new().unwrap()),
        };

        Self { runtime }
    }
}

impl ManagerService {
    pub fn spawn(&mut self, server_fut: impl Future + Send + 'static) -> JoinHandle<()> {
        // Obtain a handle to the current runtime if present.
        let handle = self
            .runtime
            .as_ref()
            .map(|rt| rt.handle().clone())
            .unwrap_or_else(Handle::current);

        handle.spawn(async move {
            server_fut.await;
        })
    }
}

#[derive(thiserror::Error, Debug, serde::Deserialize, serde::Serialize)]
pub enum WrappedPoisonError {
    #[error("cannot acquire lock on context")]
    Poison,
}

impl<T> From<PoisonError<T>> for WrappedPoisonError {
    fn from(_err: PoisonError<T>) -> Self {
        Self::Poison
    }
}

pub async fn spawn_scheduler<
    JobRequest: Clone,
    JobOutputInfo,
    Error: From<crate::Error> + From<WrappedPoisonError> + From<std::io::Error>,
>(
    result_dir: impl AsRef<Path>,
    control_context: Arc<Mutex<dyn RunContext<JobRequest, JobOutputInfo>>>,
    job: Box<dyn JobRunner<JobRequest, JobOutputInfo, Error>>,
    rest_handler: JoinHandle<()>,
) -> Result<(), Error> {
    let result_dir = result_dir.as_ref();

    let request_to_start_task = async {
        loop {
            if let Some((job_id, request)) = check_if_started(control_context.clone()) {
                let job_working_dir = result_dir.to_path_buf().join(job_id.to_string());
                std::fs::create_dir_all(&job_working_dir)?;
                control_context
                    .lock()
                    .map_err(|_| WrappedPoisonError::Poison)?
                    .new_run_started()?;
                let output_info = job.start(request, job_working_dir.to_path_buf())?;
                control_context
                    .lock()
                    .map_err(|_| WrappedPoisonError::Poison)?
                    .run_finished(output_info)?;
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
    .fuse();

    tokio::pin!(request_to_start_task);

    futures::select! {
        result = request_to_start_task => result,
        _ = rest_handler.fuse() => Ok(()),
    }
}
use crate::{JobRunner, RunContext};

fn check_if_started<JobRequest: Clone, JobOutputInfo>(
    control_context: Arc<Mutex<dyn RunContext<JobRequest, JobOutputInfo>>>,
) -> Option<(Uuid, JobRequest)> {
    if let Some((job_id, request)) = control_context.lock().unwrap().run_requested() {
        Some((job_id, request))
    } else {
        None
    }
}
