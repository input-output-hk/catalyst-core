//! EVM transactions
use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError, Serialize, WriteError},
};

/// Variants of supported EVM action types
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvmActionType {}

/// Variants of supported EVM transactions
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvmTransaction {}

impl Serialize for EvmTransaction {
    fn serialize<W: std::io::Write>(&self, _codec: &mut Codec<W>) -> Result<(), WriteError> {
        Err(WriteError::IoError(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "evm transactions are not supported in this build",
        )))
    }
}

impl Deserialize for EvmTransaction {
    fn deserialize<R: std::io::Read>(_codec: &mut Codec<R>) -> Result<Self, ReadError> {
        Err(ReadError::IoError(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "evm transactions are not supported in this build",
        )))
    }
}
