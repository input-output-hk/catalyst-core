use crate::error::{Code, Error};

use std::convert::TryFrom;

const BLOCK_ID_LEN: usize = 32;

/// Network representation of a block ID.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockId([u8; BLOCK_ID_LEN]);

pub type BlockIds = Box<[BlockId]>;

impl BlockId {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for BlockId {
    type Error = Error;

    fn try_from(src: &[u8]) -> Result<Self, Error> {
        match TryFrom::try_from(src) {
            Ok(data) => Ok(BlockId(data)),
            Err(_) => Err(Error::new(
                Code::InvalidArgument,
                format!("block identifier must be {} bytes long", BLOCK_ID_LEN),
            )),
        }
    }
}

pub fn try_ids_from_iter<I>(iter: I) -> Result<Box<[BlockId]>, Error>
where
    I: IntoIterator,
    I::Item: AsRef<[u8]>,
{
    try_ids_from_iter_desugared(iter.into_iter())
}

fn try_ids_from_iter_desugared<I>(iter: I) -> Result<BlockIds, Error>
where
    I: Iterator,
    I::Item: AsRef<[u8]>,
{
    let ids = iter
        .map(|item| BlockId::try_from(item.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids.into())
}
