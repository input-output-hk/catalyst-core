use crate::config::ConfigParam;
use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, ReadError, Serialize, WriteError},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

impl Serialize for ConfigParams {
    fn serialized_size(&self) -> usize {
        let mut res = Codec::u16_size();
        for config in &self.0 {
            res += config.serialized_size();
        }
        res
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        // FIXME: put params in canonical order (e.g. sorted by tag)?
        codec.put_be_u16(self.0.len() as u16)?;
        for config in &self.0 {
            config.serialize(codec)?
        }
        Ok(())
    }
}

impl DeserializeFromSlice for ConfigParams {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        // FIXME: check canonical order?
        let len = codec.get_be_u16()?;
        let mut configs: Vec<ConfigParam> = Vec::with_capacity(len as usize);
        for _ in 0..len {
            configs.push(ConfigParam::deserialize_from_slice(codec)?);
        }
        Ok(ConfigParams(configs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn config_params_serialize(params: ConfigParams) -> bool {
            use chain_core::property::{Serialize as _,};
            let bytes = params.serialize_as_vec().unwrap();
            let decoded = ConfigParams::deserialize_from_slice(&mut Codec::new(bytes.as_slice())).unwrap();

            params == decoded
        }

        fn config_params_serialize_readable(params: ConfigParams) -> bool {
            use chain_core::property::Serialize as _;
            let bytes = params.serialize_as_vec().unwrap();
            let decoded = ConfigParams::deserialize_from_slice(&mut Codec::new(bytes.as_slice())).unwrap();

            params == decoded
        }
    }
}
