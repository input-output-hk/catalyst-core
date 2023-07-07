use std::ops::Deref;

pub mod event;
#[cfg(feature = "jorm-mock")]
pub mod jorm_mock;
pub mod objective;
pub mod proposal;
pub mod registration;
pub mod review;
pub mod search;
pub mod voting_status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerdeType<T>(pub T);

impl<T> From<T> for SerdeType<T> {
    fn from(val: T) -> Self {
        Self(val)
    }
}

impl<T> Deref for SerdeType<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
