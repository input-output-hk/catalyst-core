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

use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;

// to remove..
pub use builder::{SetAuthData, SetIOs, SetPayload, SetWitnesses, TxBuilder, TxBuilderState};
pub use element::*;
pub use input::*;
pub use io::{Error, InputOutput, InputOutputBuilder, OutputPolicy};
pub use payload::{NoExtra, Payload, PayloadAuthData, PayloadAuthSlice, PayloadData, PayloadSlice};
pub use transaction::*;
pub use transfer::*;
pub use utxo::*;
pub use witness::*;

impl<Extra: Payload> property::Serialize for Transaction<Extra> {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.as_ref())
    }
}

impl<Extra: Payload> Readable for Transaction<Extra> {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let utx = UnverifiedTransactionSlice::from(buf.get_slice_end());
        match utx.check() {
            Ok(tx) => Ok(tx.to_owned()),
            Err(_) => Err(ReadError::StructureInvalid("transaction".to_string())),
        }
    }
}

// TEMPORARY
pub type AuthenticatedTransaction<P> = Transaction<P>;
