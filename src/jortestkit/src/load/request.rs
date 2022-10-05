use super::Status;
use rayon::iter::plumbing::{Folder, UnindexedProducer};
use std::{collections::HashMap, time::Duration};
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

pub struct Request {
    pub ids: Vec<Option<Id>>,
    pub duration: Duration,
}

/// Not a type alias because the split method is interpreted differently
/// and we make assumptions about that which we don't want to make for the general
/// type
pub trait RequestGenerator: Send + Sized {
    fn split(self) -> (Self, Option<Self>);
    fn next(&mut self) -> Result<Request, RequestFailure>;
}

pub struct RayonWrapper<T>(T);

impl<T> From<T> for RayonWrapper<T> {
    fn from(other: T) -> Self {
        Self(other)
    }
}

impl<T: RequestGenerator> UnindexedProducer for RayonWrapper<T> {
    type Item = Result<Request, RequestFailure>;
    fn split(self) -> (Self, Option<Self>) {
        let (a, b) = self.0.split();
        (Self(a), b.map(Self))
    }

    fn fold_with<F>(mut self, mut folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        while !folder.full() {
            folder = folder.consume(self.0.next());
        }
        folder
    }
}

pub trait RequestStatusProvider {
    fn get_status(&self) -> HashMap<String, RequestStatus>;
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
