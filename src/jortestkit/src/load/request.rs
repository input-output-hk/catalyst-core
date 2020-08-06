use super::Status;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RequestFailure {
    #[error("General - {0}")]
    General(String),
    #[error("Rejected - {0}")]
    Rejected(String),
}

#[derive(Debug, Copy, Clone)]
pub enum RequestSendMode {
    Sync,
    Async,
}

pub type Id = String;

pub trait RequestGenerator {
    fn next(&mut self) -> Result<Option<Id>, RequestFailure>;
}

pub trait RequestStatusProvider {
    fn get_status(&self) -> HashMap<String, RequestStatus>;
}

pub fn run_request(request: &mut impl RequestGenerator, sync: RequestSendMode) -> Response {
    match sync {
        RequestSendMode::Sync => run_request_sync(request),
        RequestSendMode::Async => run_request_async(request),
    }
}

pub fn run_request_sync(request: &mut impl RequestGenerator) -> Response {
    let start = Instant::now();
    let result = request.next();
    match result {
        Ok(id) => Response::success(id, start.elapsed()),
        Err(failure) => Response::failure(None, failure, start.elapsed()),
    }
}

pub fn run_request_async(request: &mut impl RequestGenerator) -> Response {
    let start = Instant::now();
    let result = request.next();
    match result {
        Ok(id) => Response::pending(id, start.elapsed()),
        Err(failure) => Response::failure(None, failure, start.elapsed()),
    }
}
#[derive(Debug, Clone)]
pub struct Response {
    id: Option<Id>,
    failure: Option<RequestFailure>,
    status: RequestStatus,
    duration: Duration,
}

impl Response {
    pub fn success(id: Option<Id>, duration: Duration) -> Self {
        Self {
            id,
            failure: None,
            status: RequestStatus::Success,
            duration,
        }
    }

    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    pub fn id(&self) -> &Option<Id> {
        &self.id
    }

    pub fn is_pending(&self) -> bool {
        self.status().is_pending()
    }

    pub fn is_success(&self) -> bool {
        self.status().is_success()
    }

    pub fn is_failed(&self) -> bool {
        self.status().is_failed()
    }

    pub fn status(&self) -> &RequestStatus {
        &self.status
    }

    pub fn has_id(&self, other: &str) -> bool {
        if let Some(id) = &self.id {
            return *id == *other;
        }
        false
    }

    pub fn failure(id: Option<Id>, failure: RequestFailure, duration: Duration) -> Self {
        Self {
            id,
            failure: Some(failure.clone()),
            status: RequestStatus::Failed {
                message: failure.to_string(),
            },
            duration,
        }
    }

    pub fn pending(id: Option<Id>, duration: Duration) -> Self {
        Self {
            id,
            failure: None,
            status: RequestStatus::Pending,
            duration,
        }
    }

    pub fn into_failure(self, duration: Duration) -> Self {
        Self {
            id: self.id.clone(),
            failure: self.failure.clone(),
            status: RequestStatus::Failed {
                message: self.err().as_ref().unwrap().to_string(),
            },
            duration: self.duration + duration,
        }
    }

    pub fn into_success(self, duration: Duration) -> Self {
        Self {
            id: self.id,
            failure: None,
            status: RequestStatus::Success,
            duration: self.duration + duration,
        }
    }

    pub fn new_from_status(self, status: Status) -> Self {
        Self {
            id: self.id.clone(),
            failure: status.failure(),
            status: status.status().clone(),
            duration: *self.duration() + *status.duration(),
        }
    }

    pub fn update_status(&mut self, status: Status) {
        self.failure = status.failure();
        self.status = status.status().clone();
        self.duration = *self.duration() + *status.duration();
    }

    pub fn err(&self) -> Option<RequestFailure> {
        self.failure.clone()
    }

    pub fn is_err(&self) -> bool {
        self.err().is_some()
    }

    pub fn is_ok(&self) -> bool {
        self.err().is_none()
    }
}

#[derive(Debug, Clone)]
pub enum RequestStatus {
    Failed { message: String },
    Success,
    Pending,
}

impl RequestStatus {
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Failed { .. } | Self::Success => false,
            Self::Pending => true,
        }
    }

    pub fn is_success(&self) -> bool {
        match self {
            Self::Failed { .. } | Self::Pending => false,
            Self::Success => true,
        }
    }

    pub fn is_failed(&self) -> bool {
        match self {
            Self::Pending | Self::Success => false,
            Self::Failed { .. } => true,
        }
    }
}
