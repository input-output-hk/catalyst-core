use std::ops::{Deref, DerefMut};
use cryptoxide::util::secure_memset;

#[derive(Debug, Default, Clone)]
pub struct ScrubbedBytes(Vec<u8>);

pub type Password = ScrubbedBytes;

impl From<Vec<u8>> for ScrubbedBytes {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl AsRef<[u8]> for ScrubbedBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for ScrubbedBytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl DerefMut for ScrubbedBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl Drop for ScrubbedBytes {
    fn drop(&mut self) {
        secure_memset(&mut self.0, 0)
    }
}
