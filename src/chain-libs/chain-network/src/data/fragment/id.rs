use crate::error::{Code, Error};

use std::convert::TryFrom;
use std::fmt;

const FRAGMENT_ID_LEN: usize = 32;

/// Network representation of a fragment ID.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FragmentId([u8; FRAGMENT_ID_LEN]);

pub type FragmentIds = Box<[FragmentId]>;

impl fmt::Debug for FragmentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("FragmentId(0x")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        f.write_str(")")
    }
}

impl FragmentId {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for FragmentId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl TryFrom<&[u8]> for FragmentId {
    type Error = Error;

    fn try_from(src: &[u8]) -> Result<Self, Error> {
        match TryFrom::try_from(src) {
            Ok(data) => Ok(FragmentId(data)),
            Err(_) => Err(Error::new(
                Code::InvalidArgument,
                format!("fragment identifier must be {} bytes long", FRAGMENT_ID_LEN),
            )),
        }
    }
}

pub fn try_ids_from_iter<I>(iter: I) -> Result<Box<[FragmentId]>, Error>
where
    I: IntoIterator,
    I::Item: AsRef<[u8]>,
{
    try_ids_from_iter_desugared(iter.into_iter())
}

fn try_ids_from_iter_desugared<I>(iter: I) -> Result<FragmentIds, Error>
where
    I: Iterator,
    I::Item: AsRef<[u8]>,
{
    let ids = iter
        .map(|item| FragmentId::try_from(item.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids.into())
}
