use data_pile::SharedMmap;
use sled::IVec;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
enum ValueImpl {
    Volatile(IVec),
    Owned(Box<[u8]>),
    Permanent(SharedMmap),
}

/// Wrapper for data held by the database. This wrapper holds structs returned
/// by both volatile and permanent storage to ensure we don't have needless
/// copying on return. Data should be accessed through the `AsRef` trait.
#[derive(Debug, Clone)]
pub struct Value {
    inner: ValueImpl,
}

impl Value {
    pub(crate) fn volatile(value: IVec) -> Self {
        Self {
            inner: ValueImpl::Volatile(value),
        }
    }

    pub(crate) fn owned(value: Box<[u8]>) -> Self {
        Self {
            inner: ValueImpl::Owned(value),
        }
    }

    pub(crate) fn permanent(value: SharedMmap) -> Self {
        Self {
            inner: ValueImpl::Permanent(value),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl AsRef<[u8]> for Value {
    fn as_ref(&self) -> &[u8] {
        match &self.inner {
            ValueImpl::Volatile(value) => value.as_ref(),
            ValueImpl::Owned(value) => value.as_ref(),
            ValueImpl::Permanent(value) => value.as_ref(),
        }
    }
}

impl From<Box<[u8]>> for Value {
    fn from(value: Box<[u8]>) -> Self {
        Self::owned(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self::owned(value.into_boxed_slice())
    }
}
