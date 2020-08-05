use crate::Value;
use std::io::{Read, Write};

#[derive(Clone)]
pub struct BlockInfo {
    id: Value,
    parent_id: Value,
    chain_length: u32,
    ref_count: u32,
}

impl BlockInfo {
    pub fn new<A: Into<Value>, B: Into<Value>>(id: A, parent_id: B, chain_length: u32) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            chain_length,
            ref_count: 0,
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
        self.ref_count
    }

    pub(crate) fn add_ref(&mut self) {
        self.ref_count += 1
    }

    pub(crate) fn remove_ref(&mut self) {
        self.ref_count -= 1
    }

    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut w = Vec::new();

        w.write_all(&self.chain_length.to_le_bytes()).unwrap();

        w.write_all(&self.ref_count.to_le_bytes()).unwrap();

        w.write_all(self.parent_id.as_ref()).unwrap();

        w
    }

    pub(crate) fn deserialize<R: Read, T: Into<Value>>(mut r: R, id_size: usize, id: T) -> Self {
        let mut chain_length_bytes = [0u8; 4];
        r.read_exact(&mut chain_length_bytes).unwrap();
        let chain_length = u32::from_le_bytes(chain_length_bytes);

        let mut ref_count_bytes = [0u8; 4];
        r.read_exact(&mut ref_count_bytes).unwrap();
        let ref_count = u32::from_le_bytes(ref_count_bytes);

        let mut parent_id = vec![0u8; id_size];
        r.read_exact(&mut parent_id).unwrap();

        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            chain_length,
            ref_count,
        }
    }
}
