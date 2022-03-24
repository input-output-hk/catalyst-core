mod builder;
mod element;
mod input;
mod io;
mod payload;
#[allow(clippy::module_inception)]
mod transaction;
mod transfer;
mod utxo;
mod witness;

#[cfg(any(test, feature = "property-test-api"))]
pub mod test;

use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError, Serialize, WriteError},
};

// to remove..
pub use builder::{
    SetAuthData, SetIOs, SetPayload, SetTtl, SetWitnesses, TxBuilder, TxBuilderState,
};
pub use element::*;
pub use input::*;
pub use io::{Error, InputOutput, InputOutputBuilder, OutputPolicy};
pub use payload::{NoExtra, Payload, PayloadAuthData, PayloadAuthSlice, PayloadData, PayloadSlice};
pub use transaction::*;
pub use transfer::*;
pub use utxo::*;
pub use witness::*;

impl<Extra: Payload> Serialize for Transaction<Extra> {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.as_ref())
    }
}

impl<Extra: Payload> Deserialize for Transaction<Extra> {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let mut buf = Vec::new();
        // TODO: implicitly define size of the Transaction object in the deserialize function, do not use read_to_end,
        // it narrows the usage of the deserialize trait for the Transaction struct,
        // which is not obvious from the Deserialze trait description, so leads to mistakes
        codec.read_to_end(&mut buf)?;
        let utx = UnverifiedTransactionSlice::from(buf.as_slice());
        match utx.check() {
            Ok(tx) => Ok(tx.to_owned()),
            Err(e) => Err(ReadError::StructureInvalid(e.to_string())),
        }
    }
}

// TEMPORARY
pub type AuthenticatedTransaction<P> = Transaction<P>;
