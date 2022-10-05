//! Utilities for testing the storage.

use crate::Value;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Copy)]
pub struct BlockId(pub u64);

/// Used to generate block ids
static GLOBAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

impl BlockId {
    pub fn generate() -> Self {
        Self(GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        writer.write_all(&self.0.to_le_bytes())
    }

    pub fn serialize_as_vec(&self) -> Vec<u8> {
        let mut v = Vec::new();
        self.serialize(&mut v).unwrap();
        v
    }

    pub fn serialize_as_value(&self) -> Value {
        Value::owned(self.serialize_as_vec().into_boxed_slice())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Block {
    pub id: BlockId,
    pub parent: BlockId,
    pub chain_length: u32,
    pub data: Box<[u8]>,
}

impl Block {
    pub fn genesis(data: Option<Box<[u8]>>) -> Self {
        Self {
            id: BlockId::generate(),
            parent: BlockId(0),
            chain_length: 0,
            data: data.unwrap_or_default(),
        }
    }

    pub fn make_child(&self, data: Option<Box<[u8]>>) -> Self {
        Self {
            id: BlockId::generate(),
            parent: self.id,
            chain_length: self.chain_length + 1,
            data: data.unwrap_or_default(),
        }
    }

    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        writer.write_all(&self.id.0.to_le_bytes())?;
        writer.write_all(&self.parent.0.to_le_bytes())?;
        writer.write_all(&self.chain_length.to_le_bytes())?;
        writer.write_all(&(self.data.len() as u64).to_le_bytes())?;
        writer.write_all(&self.data)?;
        Ok(())
    }

    pub fn serialize_as_vec(&self) -> Vec<u8> {
        let mut v = Vec::new();
        self.serialize(&mut v).unwrap();
        v
    }

    pub fn serialize_as_value(&self) -> Value {
        Value::owned(self.serialize_as_vec().into_boxed_slice())
    }
}
