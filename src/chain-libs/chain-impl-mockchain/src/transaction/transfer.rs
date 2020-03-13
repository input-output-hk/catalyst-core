use crate::legacy::OldAddress;
use crate::value::*;
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_ser::deser::{Deserialize, Serialize};
use chain_ser::packer::Codec;
use std::io::Error;

/// Information how tokens are spent.
/// A value of tokens is sent to the address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Output<Address> {
    pub address: Address,
    pub value: Value,
}

impl<Address: Serialize> Serialize for Output<Address> {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        let mut codec = Codec::new(writer);
        match self.address.serialize(&mut codec) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error writing Output data for Address type: {}", e),
            )),
        }?;
        codec.put_u64(self.value.0)?;
        Ok(())
    }
}

impl<Address: Deserialize> Deserialize for Output<Address> {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        let mut codec = Codec::new(reader);
        let address = match Address::deserialize(&mut codec) {
            Ok(address) => Ok(address),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Error reading Output data for Address type: {}", e),
            )),
        }?;
        let value = Value(codec.get_u64()?);
        Ok(Output { address, value })
    }
}

impl<Address: Readable> Output<Address> {
    pub fn from_address(address: Address, value: Value) -> Self {
        Output { address, value }
    }
}

impl<Address: Readable> Readable for Output<Address> {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let address = Address::read(buf)?;
        let value = Value::read(buf)?;
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
