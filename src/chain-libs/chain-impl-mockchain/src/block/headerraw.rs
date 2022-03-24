use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError, Serialize, WriteError},
};

/// Block Header Bytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderRaw(pub(super) Vec<u8>);

impl AsRef<[u8]> for HeaderRaw {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Serialize for HeaderRaw {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_be_u16(self.0.len() as u16)?;
        codec.put_bytes(&self.0)
    }
}

impl Deserialize for HeaderRaw {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let header_size = codec.get_be_u16()? as usize;
        let v = codec.get_bytes(header_size)?;
        Ok(HeaderRaw(v))
    }
}
