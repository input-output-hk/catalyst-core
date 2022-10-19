use crate::Error;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum State<
    JobRequest: Serialize + Clone,
    Step: Serialize + Clone,
    JobOutputInfo: Serialize + Clone,
> {
    Idle,
    RequestToStart {
        job_id: Uuid,
        request: JobRequest,
    },
    Running {
        job_id: Uuid,
        start: NaiveDateTime,
        request: JobRequest,
        step: Option<Step>,
    },
    Finished {
        job_id: Uuid,
        start: NaiveDateTime,
        end: NaiveDateTime,
        request: JobRequest,
        info: Option<JobOutputInfo>,
    },
    Failed {
        job_id: Uuid,
        start: NaiveDateTime,
        end: NaiveDateTime,
        request: JobRequest,
        info_msg: String,
    },
}

impl<JobRequest: Clone + Serialize, Step: Serialize + Clone, JobOutputInfo: Serialize + Clone>
    State<JobRequest, Step, JobOutputInfo>
{
    pub fn update_running_step(&mut self, new_step: Step) {
        if let State::Running { step, .. } = self {
            *step = Some(new_step);
        }
    }

    pub fn assert_is_finished(&self) {
        matches!(self, State::Finished { .. });
    }

    pub fn run_requested(&self) -> Option<(Uuid, JobRequest)> {
        match self {
            State::RequestToStart { job_id, request } => Some((*job_id, (*request).clone())),
            _ => None,
        }
    }

    pub fn new_run_requested(&mut self, request: JobRequest) -> Result<Uuid, Error> {
        match self {
            State::Idle | State::Finished { .. } => {
                let id = Uuid::new_v4();
                *self = State::RequestToStart {
                    job_id: id,
                    request,
                };
                Ok(id)
            }
            _ => Err(Error::JobInProgress),
        }
    }

    pub fn new_run_started(&mut self) -> Result<(), Error> {
        match self {
            State::RequestToStart { job_id, request } => {
                *self = State::Running {
                    job_id: *job_id,
                    start: Utc::now().naive_utc(),
                    request: request.clone(),
                    step: None,
                };
                Ok(())
            }
            _ => Err(Error::NoRequestToStart),
        }
    }

    pub fn run_finished(&mut self, info: Option<JobOutputInfo>) -> Result<(), Error> {
        match self {
            State::Running {
                job_id,
                start,
                request,
                ..
            } => {
                *self = State::Finished {
                    job_id: *job_id,
                    start: *start,
                    end: Utc::now().naive_utc(),
                    request: request.clone(),
                    info,
                };
                Ok(())
            }
            _ => Err(Error::JobNotStarted),
        }
    }

    pub fn has_id(&self, id: &Uuid) -> bool {
        match self {
            State::Idle => false,
            State::RequestToStart { job_id, .. } => job_id == id,
            State::Running { job_id, .. } => job_id == id,
            State::Finished { job_id, .. } => job_id == id,
            State::Failed { job_id, .. } => job_id == id,
        }
    }

    pub fn persist<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        use std::io::Write;
        let content = serde_yaml::to_string(&self).map_err(|e| Error::Serde(e.to_string()))?;
        let mut file = std::fs::File::create(&path).map_err(|e| Error::Io(e.to_string()))?;
        file.write_all(content.as_bytes())
            .map_err(|e| Error::Io(e.to_string()))?;
        Ok(())
    }
}

impl<
        JobRequest: Debug + Serialize + Clone,
        Step: Debug + Serialize + Clone,
        JobOutputInfo: Debug + Serialize + Clone,
    > fmt::Display for State<JobRequest, Step, JobOutputInfo>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
