use std::io::{Read, Write};

#[derive(Clone)]
pub struct BlockInfo {
    id: Box<[u8]>,
    parent_id: Box<[u8]>,
    chain_length: u32,
    ref_count: u32,
}

impl BlockInfo {
    pub fn new<T: Into<Box<[u8]>>>(id: T, parent_id: T, chain_length: u32) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            chain_length,
            ref_count: 0,
        }
    }

    pub fn id(&self) -> &[u8] {
        &self.id
    }

    pub fn parent_id(&self) -> &[u8] {
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

        let id_size = self.id.len() as u32;
        w.write_all(&id_size.to_le_bytes()).unwrap();

        let parent_id_size = self.id.len() as u32;
        w.write_all(&parent_id_size.to_le_bytes()).unwrap();

        w.write_all(&self.chain_length.to_le_bytes()).unwrap();

        w.write_all(&self.ref_count.to_le_bytes()).unwrap();

        w.write_all(&self.id).unwrap();

        w.write_all(&self.parent_id).unwrap();

        w
    }

    pub(crate) fn deserialize<R: Read>(mut r: R) -> Self {
        let mut id_size_bytes = [0u8; 4];
        r.read_exact(&mut id_size_bytes).unwrap();
        let id_size = u32::from_le_bytes(id_size_bytes);

        let mut parent_id_size_bytes = [0u8; 4];
        r.read_exact(&mut parent_id_size_bytes).unwrap();
        let parent_id_size = u32::from_le_bytes(parent_id_size_bytes);

        let mut chain_length_bytes = [0u8; 4];
        r.read_exact(&mut chain_length_bytes).unwrap();
        let chain_length = u32::from_le_bytes(chain_length_bytes);

        let mut ref_count_bytes = [0u8; 4];
        r.read_exact(&mut ref_count_bytes).unwrap();
        let ref_count = u32::from_le_bytes(ref_count_bytes);

        let mut id = vec![0u8; id_size as usize];
        r.read_exact(&mut id).unwrap();

        let mut parent_id = vec![0u8; parent_id_size as usize];
        r.read_exact(&mut parent_id).unwrap();

        Self {
            id: id.into_boxed_slice(),
            parent_id: parent_id.into_boxed_slice(),
            chain_length,
            ref_count,
        }
    }
}
