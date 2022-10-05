use crate::legacy::OldAddress;
use crate::value::*;
use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError},
};

/// Information how tokens are spent.
/// A value of tokens is sent to the address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Output<Address> {
    pub address: Address,
    pub value: Value,
}

impl<Address> Output<Address> {
    pub fn from_address(address: Address, value: Value) -> Self {
        Output { address, value }
    }
}

impl<Address: Deserialize> Deserialize for Output<Address> {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let address = Address::deserialize(codec)?;
        let value = Value::deserialize(codec)?;
        Ok(Output { address, value })
    }
}

impl std::fmt::Display for Output<chain_addr::Address> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.address.base32(), self.value)
    }
}

impl std::fmt::Display for Output<OldAddress> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.address, self.value)
    }
}
