use crate::error::{Code, Error};

use std::convert::TryFrom;

const FRAGMENT_ID_LEN: usize = 32;

/// Network representation of a block ID.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FragmentId([u8; FRAGMENT_ID_LEN]);

pub type FragmentIds = Box<[FragmentId]>;

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
