use crate::{Error, Value};
use std::io::{Read, Write};

/// A structure that holds the information about a blocks, that is needed to
/// maintain consistency of the storage. This include the ID of the blocks, the
/// ID of its parent and the length of the block chain for the given block.
#[derive(Clone)]
pub struct BlockInfo {
    id: Value,
    parent_id: Value,
    chain_length: u32,
    // These two fields are used internally by the volatile storage only. Their
    // purpose is to store the number of blocks that maintain this block as a
    // parent + the number of tags for this block. A block CANNOT be removed
    // from the volatile storage if the reference counter is greater than 1. For
    // blocks from the permanent storage this value is always equal to 1 and
    // MUST NOT be used.
    // NOTE: "removing a block" relates only to removing an abanded branch
    // entirely and does not apply to moving a block to the permanent storage.
    parent_ref_count: u32,
    tags_ref_count: u32,
}

impl BlockInfo {
    pub fn new<A: Into<Value>, B: Into<Value>>(id: A, parent_id: B, chain_length: u32) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            chain_length,
            parent_ref_count: 0,
            tags_ref_count: 0,
        }
    }

    pub fn id(&self) -> &Value {
        &self.id
    }

    pub fn parent_id(&self) -> &Value {
        &self.parent_id
    }

    pub fn chain_length(&self) -> u32 {
        self.chain_length
    }

    pub(crate) fn ref_count(&self) -> u32 {
        self.parent_ref_count + self.tags_ref_count
    }

    pub(crate) fn parent_ref_count(&self) -> u32 {
        self.parent_ref_count
    }

    pub(crate) fn add_parent_ref(&mut self) {
        self.parent_ref_count += 1
    }

    pub(crate) fn remove_parent_ref(&mut self) {
        self.parent_ref_count -= 1
    }

    pub(crate) fn add_tag_ref(&mut self) {
        self.tags_ref_count += 1
    }

    pub(crate) fn remove_tag_ref(&mut self) {
        self.tags_ref_count -= 1
    }

    pub(crate) fn serialize(&self) -> Result<Vec<u8>, Error> {
        let mut w = Vec::new();

        w.write_all(&self.chain_length.to_le_bytes())
            .map_err(Error::BlockInfoSerialize)?;

        w.write_all(&self.parent_ref_count.to_le_bytes())
            .map_err(Error::BlockInfoSerialize)?;

        w.write_all(&self.tags_ref_count.to_le_bytes())
            .map_err(Error::BlockInfoSerialize)?;

        w.write_all(self.parent_id.as_ref())
            .map_err(Error::BlockInfoSerialize)?;

        Ok(w)
    }

    pub(crate) fn deserialize<R: Read, T: Into<Value>>(
        mut r: R,
        id_size: usize,
        id: T,
    ) -> Result<Self, Error> {
        let mut chain_length_bytes = [0u8; 4];
        r.read_exact(&mut chain_length_bytes)
            .map_err(Error::BlockInfoDeserialize)?;
        let chain_length = u32::from_le_bytes(chain_length_bytes);

        let mut parent_ref_count_bytes = [0u8; 4];
        r.read_exact(&mut parent_ref_count_bytes)
            .map_err(Error::BlockInfoDeserialize)?;
        let parent_ref_count = u32::from_le_bytes(parent_ref_count_bytes);

        let mut tags_ref_count_bytes = [0u8; 4];
        r.read_exact(&mut tags_ref_count_bytes)
            .map_err(Error::BlockInfoDeserialize)?;
        let tags_ref_count = u32::from_le_bytes(tags_ref_count_bytes);

        let mut parent_id = vec![0u8; id_size];
        r.read_exact(&mut parent_id)
            .map_err(Error::BlockInfoDeserialize)?;

        Ok(Self {
            id: id.into(),
            parent_id: parent_id.into(),
            chain_length,
            parent_ref_count,
            tags_ref_count,
        })
    }
}
