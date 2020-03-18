use crate::config::ConfigParam;
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;
use chain_ser::deser::Deserialize;
use std::io::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "generic-serialization",
    derive(serde_derive::Serialize, serde_derive::Deserialize),
    serde(transparent)
)]
pub struct ConfigParams(pub(crate) Vec<ConfigParam>);

impl ConfigParams {
    pub fn new() -> Self {
        ConfigParams(Vec::new())
    }

    pub fn push(&mut self, config: ConfigParam) {
        self.0.push(config)
    }

    pub fn iter(&self) -> std::slice::Iter<ConfigParam> {
        self.0.iter()
    }
}

impl property::Serialize for ConfigParams {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        // FIXME: put params in canonical order (e.g. sorted by tag)?
        use chain_core::packer::*;
        Codec::new(&mut writer).put_u16(self.0.len() as u16)?;
        for config in &self.0 {
            config.serialize(&mut writer)?
        }
        Ok(())
    }
}

impl property::Deserialize for ConfigParams {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        use chain_core::packer::Codec;
        let mut codec = Codec::new(reader);
        let size = codec.get_u16()? as usize;
        let mut config_params: Vec<ConfigParam> = Vec::with_capacity(size);
        for _ in 0..size {
            config_params.push(ConfigParam::deserialize(&mut codec)?);
        }
        Ok(ConfigParams(config_params))
    }
}

impl Readable for ConfigParams {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        // FIXME: check canonical order?
        let len = buf.get_u16()?;
        let mut configs = vec![];
        for _ in 0..len {
            configs.push(ConfigParam::read(buf)?);
        }
        Ok(ConfigParams(configs))
    }
}
